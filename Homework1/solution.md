## My notes
The co-founder of Celestia, Mustafa Al-Bassam, provided technical insights into the mechanics of the platform in a podcast.

Celestia is a B2B infrastructure that enables developers to run their own blockchains without worrying about network data availability or consensus. While it doesn't have an execution layer that allows you to deploy or execute smart contracts directly, Rollkit can be used to create a new execution layer (i.e., blockchain) with ease.

Interoperability between blockchains built with Cosmos SDK can be challenging as blindly relying on the other network's validators may not be secure in a world with thousands or millions of chains. Celestia solves this problem by ensuring you interoperate with other blockchains using the same set of trusted validators.

Celestia doesn't offer tools for blockchain interoperability between created blockchains, though. Mustafa stated that it's the responsibility of the execution layers allowing developers to decide for themselves which protocol to use to make the blockchains interoperable.
