## My notes
John Adler provided some insights on the topic of Optimistic vs ZK rollups.

The oversimplified difference between Optimistic and ZK rollups is the difference in proofs. In ZK rollups, you use validity proofs; in Optimistic, you use fraud proofs.

Plasma and State Channels use fraud proofs, too. The difference between those and Optimistic is that the latter has open participation, doesn't have liquidity constraints, is permissionless, and uses general-purpose fraud proofs.

It's fun to listen to John when he talks about large general-purpose circuits because nowadays, there are few of them in production implementing the entire EVM. Back in 2020, only some specific-purpose ZKP rollups were designed to work with DeFi or NFTs.
Also, it's interesting how he talks about only one Prover in ZKP and trust assumptions because this is also a common pattern now.

He also discussed the interesting idea that rollups can act similarly to sharding. E.g., somebody can run 64 rollups, and you can choose only a few of them to validate if you use them. It looks like what OP Stack is doing.