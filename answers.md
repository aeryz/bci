# 1, 2

Check the source code.

# 4

See `examples/traverse_dir.bci` and run it by executing `cargo r --example runner -- examples/traverse_dir.bci`

It will traverse through all files under examples folder, and its subfolders recursively and will print the line count of each `rs` file.

# 3

We need to deal with the async calls and instructions. To achieve this, we need to have an executor, which will generate state-machine like execution flow from the bytecoed and poll each part of the function until it drives them to completion. Something like this:

```
ASYNC_FN_1:
LOAD_VAL 1
LOAD_VAL 2
ASYNC_CALL SOME_IO_OP_1
ASYNC_CALL SOME_IO_OP 2
RETURN
```

which can be interpreted as:

```
ASYNC_FN_1_P1:
LOAD_VAL 1
LOAD_VAL 2

ASYNC_FN_1_P2:
POLL SOME_IO_OP_1

ASYNC_FN_1_P3:
POLL SOME_IO_OP_2
```

The executor should keep track of the state and jump to corresponding part. By using this mechanism we can poll `SEND`, `RECEIVE` and other operations without blocking each other.

# 5 

For example, SHA-256. It takes any amount of data and outputs 256 bits of data. This enables efficient verifications like:
- Instead of checking every element of block to see if it has been altered or not, we hash the block and compare just 256 bits.
- Instead of signing very very long data (which is super inefficient and slow actually because we will have to use this very long data as an exponent) we just hash the data and sign the output hash.
- In bitcoin for example, instead of hashing all the transactions at once, merkle hash tables are used which produce a single root and makes the validation process much more efficient.


# 6

Using bitcoin, to transfer some amount, you don't actually transfer the exact amount. You have some unspent transaction outputs and spend some of them and get new UTXO. If you have a 3 bitcoin worth of UTXO and you want to spend 2 bitcoin, you spend that 3 bitcoin worth of UTXO and get 1 bitcoin worth of UTXO back.

Nodes know the state of an UTXO, from the beginning of its creation so that they can verify if it is spent previously or not. And public key cryptography (sigscript, pubkey etc) is used to verify the user who spents the UTXO has the right to do so.

# 7

What makes the bitcoin a blockchain is each block contains the hash of the previous block. This way, if a block is changed, the block hash and all of the following hashes will be also change. Therefore the block until that point will be invalid.

There is a merkle root of transactions which is merkle hash of all transactions. This makes it efficient to validate the integrity of data.

There is a difficulty field, which enables miners to make it more difficult to mine a block. And it is increased every some amount of years. It is basically the number of zeros at the beginning of the block hash. Since the output of SHA-256 is not predictable, miners need to do brute-forcing. This is the thing that actually enables POW.

# 8

Distributed systems like bitcoin, etherium, etc. need to reach some kind of consensus. Because there are millions of participants who needs to see the same data at a given time (they eventually do). 

On bitcoin, it is achieved by Proof-of-Work consensus algorithm. It makes it so hard to find a valid hash for a block so that at a time theta, only very little amount of participants can produce a block. And other nodes follow them. This results in forking, but the blocks are eventually finalized. 

Proof-of-stake based consensus algorithms uses validators to produce a block. These validators are selected each round based on the amount of stake they have. The higher you stake, higher your chance.

From the energy consumpsion perspective, PoW requires huge amount of computing power because the same operation is repeated countless of times. PoS can be considered as a "green" consensus algotihm since it doesn't require mining.

From the security perspective, when the bitcoin is forked, eventually the most validated/used fork is being considered as the correct chain. So you need to at least 51% of the participants to break the whole system. But the proof-of-stake based algorithms are generally BFT, so this drops the number to 1/3.

