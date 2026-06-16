const { CathedralAgent } = require('./cathedral-napi/target/debug/cathedral_napi.node');

async function test() {
    let agent = new CathedralAgent();
    await agent.tick();
    console.log(agent.currentRound());
}
test();
