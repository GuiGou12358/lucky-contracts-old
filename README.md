# Lucky-contracts
Smartcontracts to distribute the rewards received by the developer from dAppStaking.
Based on the configuration (ratioDistribition) 100%, 80%, ... of rewards will be distributed randomly to 1,2,3, ... lucky participant(s).


Structure of the project:
<pre>
 |-- contracts/
 |   |-- dapps_staking_developer/
 |       |-- lib.rs
 |   |-- lucky_oracle/
 |       |-- lib.rs
 |   |-- lucky_raffle/
 |       |-- lib.rs
 |   |-- reward_manager/
 |       |-- lib.rs
 |-- traits/
 |   |-- reward
 |       |-- psp22_reward.rs
 |   |-- oracle.rs
 |   |-- raffle.rs
 |   |-- random_generators.rs
 |-- logics/
 |   |-- helpers/
 |       |-- helper.rs
 |   |-- impls/
 |       |-- reward
 |           |-- psp22_reward.rs
 |       |-- oracle.rs
 |       |-- raffle.rs    
 |       |-- random_generators.rs
 |-- tests/
 |   |-- oracle.rs
 |   |-- psp22_reward.rs   
 |   |-- raffle.rs
 </pre>
 
## Smart contract 'dAppStaking Developer'

This smart contract will be registrered as developer in the dAppStaking module and will receive rewards from dAppStaking.
The smart contract 'Raffle' will be whitelisted to be able to withdraw these rewards.

### Build the contract ###
```bash
cd contracts/dapps_staking_developer
cargo +nightly contract build
```

## Smart contract 'lucky Oracle'

This smart contract will act as an Oracle to provide the following data:
 - list of participants who stake/vote for this dApp in dAppStaking module
 - rewards received from dAppStaking (developer rewards)  

### Build the contract ###
```bash
cd contracts/lucky_oracle
cargo +nightly contract build
```

## Smart contract 'reward Manager'

This smart contract will manage rewards to distribute to the lucky addresses

### Build the contract ###
```bash
cd contracts/reward_manager
cargo +nightly contract build
```

## Smart contract 'lucky Raffle'

This smart contract will :
 - read the data from the oracle
 - randomly select address(es) in the list of participants
 - transfer the fund from 'dAppStacking developer' to 'reward Manager' contracts
 - set the lucky address(es) in the 'reward Manager' contract  

### Build the contract ###
```bash
cd contracts/lucky_raffle
cargo +nightly contract build
```


## Runs the tests

```bash
cargo +nightly test
```




