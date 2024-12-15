"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const { ApiPromise, WsProvider } = require('@polkadot/api');
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const local_url = "ws://127.0.0.1:9944";
const main = () => __awaiter(void 0, void 0, void 0, function* () {
    const wsProvider = new WsProvider(local_url);
    const api = yield ApiPromise.create({
        provider: wsProvider,
        types: {
            PostByPointer: {
                id: 'u32',
                content: 'Text',
                author: 'AccountId',
                timestamp: 'Moment'
            }
        },
    });
    yield api.isReady;
    const app = (0, express_1.default)();
    app.use((0, cors_1.default)());
    app.get('/api/postsByDate', (req, res) => __awaiter(void 0, void 0, void 0, function* () {
        const now_time = yield api.query.timestamp.now();
        console.log(`Current time: ${now_time}`);
        const now_date = Math.floor(now_time.toNumber() / 86400000);
        console.log(`Current date: ${now_date}`);
        // 通过日期键访问 PostsByPostDate 存储项
        const postVec = yield api.query.bbs.postsByPostDate(now_date);
        console.log(`PostVec: ${postVec}`);
        console.log(`PostVec type: ${typeof postVec}`);
        if (postVec.isNone) {
            res.status(404).send('No posts found for the given date');
            return;
        }
        const posts = [];
        const postPointers = postVec.unwrap();
        for (const postPointer of postPointers) {
            const post = yield api.query.bbs.postsByPointer(postPointer);
            if (post.isSome) {
                posts.push(post.unwrap());
            }
        }
        const htmlContent = `
            <!DOCTYPE html>
            <html>
            <head>
                <title>Posts By Date</title>
            </head>
            <body>
                <h1>Posts By Date</h1>
                ${posts.map(post => `
                    <div>
                        <h2>Post ID: ${post.id}</h2>
                        <p>Content: ${Buffer.from(post.content.slice(2), 'hex').toString()}</p>
                        <p>Author: ${post.owner}</p>
                        <p>Likes: ${post.likes}</p>
                        <p>Dislikes: ${post.dislikes}</p>
                        <p>Attention: ${post.attention}</p>
                        <p>Posted Time: ${new Date(post.postedTime).toLocaleString()}</p>
                        <p>Last Reply Time: ${new Date(post.lastReplyTime).toLocaleString()}</p>
                    </div>
                `).join('')}
            </body>
            </html>
        `;
        res.send(htmlContent);
    }));
    app.get('/', (req, res) => {
        res.sendFile(__dirname + '/index.html');
    });
    app.listen(3000, () => {
        console.log('Server is running on http://localhost:3000');
    });
});
main()
    .then(() => {
    console.log("Finished");
    process.exit(0);
})
    .catch((error) => {
    console.log("Error: ", error);
    process.exit(1);
});
//# sourceMappingURL=app.js.map