
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;



#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::marker::PhantomData;
    use frame_support::traits::GenesisBuild;
    //use sp_state_machine::*;
    //use frame_support::debug;
    use log::warn;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config + TypeInfo {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }


    /// ## Events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has been created. \[who\]
        UserCreated(T::AccountId),
        /// A user has been updated. \[who\]
        UserUpdated(T::AccountId),
        /// A post has been created. \[who, post_id\]
        PostCreated(T::AccountId, (T::Moment, T::AccountId)),
        /// A post has been liked. \[who, post_id\]
        PostLiked(T::AccountId, (T::Moment, T::AccountId)),
        /// A post has been disliked. \[who, post_id\]
        PostDisliked(T::AccountId, (T::Moment, T::AccountId)),
        /// A post has been attended. \[who, post_id\]
        PostAttended(T::AccountId, (T::Moment, T::AccountId)),
        /// A private message has been sent. \[from, to\]
        PrivateMsg(T::AccountId, T::AccountId),
    }

    /// ## Errors
    #[pallet::error]
    pub enum Error<T> {
        /// The nickname is too large.
        NicknameTooLarge,
        /// The user has changed the information too often.
        ChangeInfoTooOften,
        /// The user is not registered.
        UnregisteredUser,
        /// The content is too large.
        ContentTooLarge,
        /// The post is too frequent.
        PostTooFrequent,
        /// The post is not found.
        PostNotFound,
        /// The user likes their own post.
        LikeOwnPost,
        /// The user dislikes their own post.
        DislikeOwnPost,
        /// The user attends their own post.
        AttentionOwnPost,
        /// The receiver is not registered.
        ReceiverUnregistered,
        /// The message is too large.
        MsgTooLarge,
        /// The public key is invalid.
        InvalidPublicKey,
        /// The encryption failed.
        EncryptionFailed,
        Overflow,
        /// Too many posts on this date.
        TooManyPostsOnThisDate,
        /// Too much reply on this post.
        TooMuchReplyOnThis,
    }

    /// ## User System

    #[pallet::storage]
    #[pallet::getter(fn posts)]
    pub type UserSum <T: Config> = StorageValue<_, u128>; // the sum of all user.

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo)]
    pub struct UserSystemItem<T: Config> {
        ///**lastChange**: the last time the user changed their information (to limit the frequency of changes)
        pub lastChange: T::Moment,
        /// **nickname**: a string with a maximum length of 64 characters
        pub nickname: BoundedVec<u8, ConstU32<64>>,
        /// **avatar**: hash string of the Gravatar avatar
        pub avatar: BoundedVec<u8, ConstU32<32>>,
        /// **rsaPublicKey**: the public key of the user's RSA encryption algorithm, use for private message
        pub rsaPublicKey: BoundedVec<u8, ConstU32<256>>,
        // TODO: Regulation
        // pub bannedUntilTime: T::Moment,
    }

    #[pallet::storage]
    #[pallet::getter(fn users)]
    pub type Users<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserSystemItem<T>>;
    //

    /// ### Interaction with the user system

    


    /// **register_or_change_user_info**: Register a new user or change the user's information.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_or_change_user_info(origin: OriginFor<T>, nickname: BoundedVec<u8, ConstU32<64>>, avatar: BoundedVec<u8, ConstU32<32>>, rsaPublicKey: BoundedVec<u8, ConstU32<256>>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Check the length of the nickname to ensure it does not exceed the limit
            ensure!(nickname.len() <= 64, Error::<T>::NicknameTooLarge);
            // if the user is not registered, register the user, otherwise update the user information
            if(Users::<T>::contains_key(&who)){
                let mut user = Users::<T>::get(&who);
                // can only change info once a week
                let mut userr = user.unwrap();
                // check if the change is too frequent
                /*
                ensure!(
                    pallet_timestamp::Pallet::<T>::get() - userr.lastChange >= T::Moment::from(604800u64),
                    Error::<T>::ChangeInfoTooOften
                );
                */
                // get the last change time
                userr.lastChange = pallet_timestamp::Pallet::<T>::get();
                userr.nickname = nickname;
                userr.avatar = avatar;
                userr.rsaPublicKey = rsaPublicKey;
                Users::<T>::insert(&who, userr);
                Self::deposit_event(Event::UserUpdated(who));
                
            }else{
                // check if overflow
                let sum = UserSum::<T>::get().unwrap();
                let new_sum = sum.saturating_add(1);
                ensure!(new_sum > sum, Error::<T>::Overflow);
                UserSum::<T>::put(new_sum);
                let user = UserSystemItem{
                    lastChange: pallet_timestamp::Pallet::<T>::get(),
                    nickname: nickname,
                    avatar: avatar,
                    rsaPublicKey: rsaPublicKey,
                };
                Users::<T>::insert(&who, user);
                Self::deposit_event(Event::UserCreated(who));
            }
            Ok(())
        } 
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        /// **fn new_post**: Create a new post. (if reply_to is not None, it is a reply, otherwise it will open a new thread)
        pub fn new_post(
            origin: OriginFor<T>,
            content: BoundedVec<u8, ConstU32<2048>>,
            reply_to: Option<(T::Moment, T::AccountId)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check if the caller is registered
            ensure!(Users::<T>::contains_key(& who), Error::<T>::UnregisteredUser);
            // check the length
            warn!("content.len() = {}", "content.len()");

            ensure!(content.len() <= 2048, Error::<T>::ContentTooLarge);
            let now = pallet_timestamp::Pallet::<T>::get();
            let post_id = (now, who.clone());
            ensure!(!PostsByPointers::<T>::contains_key(&post_id), Error::<T>::PostTooFrequent);
            //warn!("post_id = {:?}", post_id);

            let new_post = Post {
                content: content,
                owner: who.clone(),
                replies: BoundedVec::default(),
                likes: 0,
                dislikes: 0,
                attention: 0,
                postedTime: now,
                lastReplyTime: now,
            };

            //warn!("new_post = {:?}", new_post);

            // check the pointer is unique
            PostsByPointers::<T>::insert(post_id.clone(), new_post);
            
            let post_date = TryInto::<u64>::try_into(now).ok().unwrap()/ 86400000;
            PostsByPostDate::<T>::try_mutate(post_date, |list| {
                if list.is_none() {
                    *list = Some(BoundedVec::default());
                }
                list.as_mut()
                    .unwrap()
                    .try_push(post_id.clone())
                    .map_err(|_| Error::<T>::TooManyPostsOnThisDate)
            })?;

            //warn!("post_date = {}", post_date);

            // add 1 to the monitoring system
            let mut monitored_day = MonitoredDay::<T>::get();
            if monitored_day == Some(post_date)  {
                Monitoring::<T>::mutate(|n| {
                    if let Some(n) = n {
                        *n = n.saturating_add(1);
                    } else {
                        *n = Some(1);
                    }
                });
            } else {
                Monitoring::<T>::put(1);
                MonitoredDay::<T>::put(post_date);
            }

            // reply
            if let Some(reply_to_id) = reply_to {
                let reply_to_id_clone = reply_to_id.clone();
                // get the post to reply to
                ensure!(PostsByPointers::<T>::contains_key(&reply_to_id), Error::<T>::PostNotFound);
                let old_post = PostsByPointers::<T>::get(&reply_to_id).unwrap();
                let old_time = old_post.lastReplyTime;
                // push could be failed due to bound, use try_push, if err raise TooMuchReplyOnThis
                PostsByPointers::<T>::try_mutate(reply_to_id, |p| -> Result<(), Error<T>> {
                    if let Some(post) = p {
                        post.replies.try_push(post_id.clone()).map_err(|_| Error::<T>::TooMuchReplyOnThis)?;
                        post.lastReplyTime = now;
                    }
                    Ok(())
                })?;
                
                // update the index
                let reply_date = post_date;
                PostsByLastReplyDate::<T>::try_mutate(reply_date, |list| {
                    if let Some(list) = list {
                        list.try_push(post_id.clone()).map_err(|_| Error::<T>::TooManyPostsOnThisDate)
                    } else {
                        *list = Some(BoundedVec::default());
                        list.as_mut().unwrap().try_push(post_id.clone()).map_err(|_| Error::<T>::TooManyPostsOnThisDate)
                    }
                })?;
                // delete old record in PostsByLastReplyDate
                let old_reply_date = TryInto::<u64>::try_into(old_time).ok().unwrap()/ 86400000;
                PostsByLastReplyDate::<T>::mutate(old_reply_date, |list| {
                    list.as_mut().unwrap().retain(|x| x != &reply_to_id_clone);
                });
            } // if new posted, replytime = posttime, insert into PostsByLastReplyDate
            else {
                PostsByLastReplyDate::<T>::mutate(post_date, |list| {
                    if list.is_none() {
                        *list = Some(BoundedVec::default());
                    }
                    list.as_mut().unwrap().try_push(post_id.clone()).map_err(|_| Error::<T>::TooManyPostsOnThisDate)
                })?;
            }
            Self::deposit_event(Event::PostCreated(who, post_id));
            Ok(())
        }
        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        /// **fn like_post**: Like a post.
        pub fn like_post(origin: OriginFor<T>, post_id: (T::Moment, T::AccountId)) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check if the caller is registered
            ensure!(Users::<T>::contains_key(& who), Error::<T>::UnregisteredUser);
            // check if the post exists
            ensure!(PostsByPointers::<T>::contains_key(&post_id), Error::<T>::PostNotFound);
            // check if the post is not liked by the owner
            let post = PostsByPointers::<T>::get(&post_id).unwrap();
            ensure!(post.owner != who, Error::<T>::LikeOwnPost);
            // update the post
            let post_id_clone = post_id.clone();
            PostsByPointers::<T>::mutate(post_id_clone, |p| {
                if let Some(post) = p {
                    post.likes = post.likes.saturating_add(1);
                }
            });
            Self::deposit_event(Event::PostLiked(who, post_id));
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        /// **fn dislike_post**: Dislike a post.
        pub fn dislike_post(origin: OriginFor<T>, post_id: (T::Moment, T::AccountId)) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check if the caller is registered
            ensure!(Users::<T>::contains_key(& who), Error::<T>::UnregisteredUser);
            // check if the post exists
            ensure!(PostsByPointers::<T>::contains_key(&post_id), Error::<T>::PostNotFound);
            // check if the post is not disliked by the owner
            let post = PostsByPointers::<T>::get(&post_id).unwrap();
            ensure!(post.owner != who, Error::<T>::DislikeOwnPost);
            // update the post
            let post_id_clone = post_id.clone();
            PostsByPointers::<T>::mutate(post_id_clone, |p| {
                if let Some(post) = p {
                    post.dislikes = post.dislikes.saturating_add(1);
                }
            });
            Self::deposit_event(Event::PostDisliked(who, post_id));
            Ok(())
        }
        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        /// **fn attention_post**: Attention a post.
        pub fn attention_post(origin: OriginFor<T>, post_id: (T::Moment, T::AccountId)) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check if the caller is registered
            ensure!(Users::<T>::contains_key(& who), Error::<T>::UnregisteredUser);
            // check if the post exists
            ensure!(PostsByPointers::<T>::contains_key(&post_id), Error::<T>::PostNotFound);
            // check if the post is not attended by the owner
            let post = PostsByPointers::<T>::get(&post_id).unwrap();
            ensure!(post.owner != who, Error::<T>::AttentionOwnPost);
            // update the post
            let post_id_clone = post_id.clone();
            PostsByPointers::<T>::mutate(post_id_clone, |p| {
                if let Some(post) = p {
                    post.attention = post.attention.saturating_add(1);
                }
            });
            Self::deposit_event(Event::PostAttended (who, post_id));
            Ok(())
        }
        
    }



    /// ## Monitoring System
    #[pallet::storage]
    #[pallet::getter(fn monitoring)]
    // **monitoring**: a u128 value that represents new post count in the last 24 hours
    // iniitial value is 0
    pub type Monitoring<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn monitored_day)]
    // **monitored_day**: a u32 value that represents the day that the monitoring system is monitoring
    pub type MonitoredDay<T: Config> = StorageValue<_, u64>;


    // private function add_1 only be called by the fucntion **new_post**
    // saturated add 1 to monitored_day if in the same day, otherwise set it to 1


    /// ## Post Database
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo)]
    pub struct Post<T: Config> {
        pub content: BoundedVec<u8, ConstU32<2048>>,
        pub owner: T::AccountId,
        pub replies: BoundedVec<(T::Moment, T::AccountId), ConstU32<256>>,
        pub likes: u128,
        pub dislikes: u128,
        pub attention: u128,
        pub postedTime: T::Moment,
        pub lastReplyTime: T::Moment,
    }

    /// **PostByTimeAndSender**: A map that stores all posts.
    #[pallet::storage]
    #[pallet::getter(fn post_database)]
    pub type PostsByPointers<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::Moment>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        Post<T>,
    >;
    
    /// **PostByPostDate**: an index that stores all posts by date.
    #[pallet::storage]
    #[pallet::getter(fn posts_index_post_date)]
    pub type PostsByPostDate<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // Date precise to day
        BoundedVec<(T::Moment, T::AccountId), ConstU32<10000>>, // List of pointers to Post objects
    >;

    /// **PostByLastReplyDate**: an index that stores all posts by the last reply date.
    #[pallet::storage]
    #[pallet::getter(fn posts_index_reply_date)]
    pub type PostsByLastReplyDate<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // Date precise to day
        BoundedVec<(T::Moment, T::AccountId), ConstU32<10000>>, // List of pointers to Post objects
    >;


    
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub user_sum: Option<u128>,
        pub monitoring: Option<u128>,
        pub monitored_day: Option<u64>,
        pub _phantom: PhantomData<T>,
    }


    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                user_sum: Some(0),
                monitoring: Some(0),
                monitored_day: Some(0),
                _phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(user_sum) = self.user_sum {
                UserSum::<T>::put(user_sum);
            }
            if let Some(monitoring) = self.monitoring {
                Monitoring::<T>::put(monitoring);
            }
            if let Some(monitored_day) = self.monitored_day {
                MonitoredDay::<T>::put(monitored_day);
            }
        }
    }
    

}
