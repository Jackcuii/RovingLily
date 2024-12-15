"use strict";
const { ApiPromise, WsProvider } = require('@polkadot/api');
const fs = require('fs');
const path = require('path');
const local_url = "ws://127.0.0.1:9944";
const main = async () => {
    const wsProvider = new WsProvider(local_url);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            // 定义PostByPointer的类型
            PostByPointer: {
                id: 'u32',
                content: 'Text',
                author: 'AccountId',
                timestamp: 'Moment'
            }
        },
    });
    await api.isReady;
    // 假设我们要查询的PostByPointer的键是1
    const now_time = await api.query.timestamp.now();
    console.log(`Current time: ${now_time}`);
    const now_date = Math.floor(now_time.toNumber() / 86400000);
    console.log(`Current date: ${now_date}`);
    const postVec = await api.query.bbs.postsByPostDate(now_date);
    console.log(`PostVec: ${postVec}`);
    console.log(`PostVec type: ${typeof postVec}`);
    const posts = [];
    for (const postPointer of postVec.unwrap()) {
        const post = await api.query.bbs.postsByPointers(postPointer[0], postPointer[1]);
        if (post.isSome) {
            console.log(`Post: ${post.unwrap()}`);
            posts.push(post.unwrap());
        }
    }
    // Create HTML content
    // Create HTML content
    let htmlContent = `
<html>
<head>
    <title>Roving Lily 漂流百合 DEMO</title>
    <link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap" rel="stylesheet">
    <style>
        body {
            font-family: 'Garamond', sans-serif;
            background-color: #f4f4f9;
            color: #333;
            margin: 0;
            padding: 20px;
        }
        h1, h2 {
            text-align: center;
        }
        ul {
            list-style-type: none;
            padding: 0;
        }
        h3 {
            color: #007BFF;
        }
        p {
            font-size: 1.1em;
        }
        hr {
            border: 0;
            height: 1px;
            background: #ddd;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <h1>Roving Lily 漂流百合 (DEMO)</h1>
    <h2>Where roving lily travels, where it take roots.</h2>
    <ul>`;
    for (const post of posts) {
        const owner = post.owner;
        const user = await api.query.bbs.users(owner);
        const nickname = user.isSome ? new TextDecoder().decode(user.unwrap().nickname) : 'Unknown';
        const content = new TextDecoder().decode(post.content);
        htmlContent += `<li><h3>${nickname} Says</h3>`;
        htmlContent += `<p>${content}</p>`;
        htmlContent += `<p>👍 ${post.likes}    👎 ${post.dislikes}    👀 ${post.attention}</p>`;
        htmlContent += '<hr></li>';
    }
    htmlContent += `
    </ul>
</body>
</html>`;
    // Write HTML content to file
    const filePath = path.join(__dirname, 'posts.html');
    fs.writeFileSync(filePath, htmlContent, 'utf8');
    console.log(`Posts have been written to ${filePath}`);
};
main()
    .then(() => {
    console.log("Finished");
    process.exit(0);
})
    .catch((error) => {
    console.log("Error: ", error);
    process.exit(1);
});
