# Loto
My first smartcontract to learn Rust and Wasm!

I try to create a lib to redistribute randomly (or not) some rewards with the main idea:
  - the rewards can come from several sources: dAppStaking, vault or strategy in DeFi, ...
  - the rewards can be any fungible or non-fungible token
  - the rewards can be distributed to all participants or only a subset of participants
  - the accounts receiving the rewards can be randomly selected or set by another smartcontract (the winner of a game, ...)
  
In this first implementation, rewards and participants are set manually. The winners (accounts receiving the rewards) are randomly selected.
