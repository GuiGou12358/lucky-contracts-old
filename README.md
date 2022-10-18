# Loto
My first smartcontract to learn Rust and Wasm!

I try to create a lib to redistribute randomly (or not) some rewards with the main idea:
  - the rewards can come from several sources: dAppStaking, vault or strategy in DeFi, ...
  - the rewards can be any fungible or non-fungible token
  - the rewards can be distributed to all participants or only a subset of participants
  - the accounts receiving the rewards can be randomly selected or set by another smartcontract (the winner of a game, ...)
  
In this first implementation, rewards and participants are set manually. The winners (accounts receiving the rewards) are randomly selected.

Structure of the lib:
<pre>
 |-- traits/
 |   |-- participant_management.rs
 |   |-- rafle.rs    
 |   |-- game.rs
 |   |-- reward/
 |       |-- reward.rs
 |       |-- psp22_reward.rs
 |-- impls/
 |   |-- manual_participant_management.rs
 |   |-- rafle.rs    
 |   |-- game.rs
 |   |-- reward/
 |       |-- psp2/
 |           |-- psp22_reward.rs
 |           |-- native_psp22_reward.rs
 |-- tests/
 |   |-- manual_participant_management.rs
 |   |-- native_psp22_reward.rs   
 |   |-- game.rs
 </pre>
 
 First contract: contract/contract_1
