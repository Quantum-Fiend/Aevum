// Example: Async Node.js program with Aevum tracing

const aevum = require('../agents/node-agent/agent');

async function fetchData(id) {
    // Simulate async operation
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve({ id, data: `Data for ${id}` });
        }, Math.random() * 1000);
    });
}

async function processData(items) {
    const results = [];
    for (const item of items) {
        const data = await fetchData(item);
        results.push(data);
        console.log(`Processed: ${data.data}`);
    }
    return results;
}

async function main() {
    console.log('🎬 Starting async Node.js program with Aevum tracing...');

    // Attach the Aevum agent
    await aevum.attach('async-example', 'localhost', 9876);

    // Process items asynchronously
    const items = [1, 2, 3, 4, 5];
    const results = await processData(items);

    console.log(`\n✅ Processed ${results.length} items`);
    console.log('Trace captured async/await execution flow!');

    // Detach the agent
    aevum.detach();
}

main().catch(console.error);
