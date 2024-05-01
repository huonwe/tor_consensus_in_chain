# tor_consensus_in_chain
# 使用substrate 构建区块链，并获取tor共识信息，将其储存在链上。
# 编译指令
```bash
cd tor_consensus_in_chain
cargo build --release
./target/release/solochian-template-node
# get tor consensus
curl http://localhost:8080/update
```
