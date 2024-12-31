# Rake
Rake (named after the tool used for lockpicking) is a blazingly fast Rust-based tool for brute forcing 4-byte EVM function signatures. Built for decompilation in mind, Rake is designed to be able to aid in reverse engineering the original function signature for methods in black box smart contracts.

## Installing
TBD.

## Modes
At this current moment, Rake only supports a single mode that involves taking in an input of a function's pre-existing arguments and attempting to reverse the selector hash through generating a random sequence of words using the supplied dictionary as reference. In the future, Rake will support additional means of brute forcing function signatures such as through the use of generating random arguments for the function.

### Args Mode
Here is a sample of Rake being used to brute force the signature of the ERC20 ``transferFrom(address,address,uint256)`` method through the use of pre-existing args and a dictionary containing token transfer related terminology:
**CLI Input:**
```
rake -d transfer,to,from,tokens,user -a "address,address,uint256" -m 23b872dd
```
**Output:**
```
Iterating over 3628800 total possibilities or factorial(10)
function user(address,address,uint256) => 95efadd4
function transfer(address,address,uint256) => beabacc8
function tokens(address,address,uint256) => f56e81fa
function to(address,address,uint256) => d254bc3e
function from(address,address,uint256) => ebd67962
function transferToFrom(address,address,uint256) => 9a972a4e
function transferToUserTokens(address,address,uint256) => 69b71ca1
function transferTo(address,address,uint256) => a5f2a152
function tokens(address,address,uint256) => f56e81fa
function userToFromTokens(address,address,uint256) => 3a51a709
function to(address,address,uint256) => d254bc3e
function userToFrom(address,address,uint256) => 472ecfbe
function transferToFromUser(address,address,uint256) => a572a655
function transferToUser(address,address,uint256) => fa93b2a5
function transferTokensFrom(address,address,uint256) => 52912042
function transferTokensUser(address,address,uint256) => f8a8e818
function tokensTo(address,address,uint256) => 16e641b1
function transferToFromTokens(address,address,uint256) => 1a9e55b2
function userTokens(address,address,uint256) => 0408f767
function transferTokens(address,address,uint256) => a64b6e5f
Found target function signature for 23b872dd: transferFrom(address,address,uint256)
```
## CLI Usage
```
Usage: rake [OPTIONS] --args <FUNC_ARGS>

Options:
  -d, --dictionary <DICTIONARY>
          List of known words to use in an attempt to brute force a matching signature
  -a, --args <FUNC_ARGS>
          Arguments of the function being brute forced, used for constructing a valid signature
  -m, --match-selector <MATCH_SELECTOR>
          Selector to attempt to create a matching signature for [default: 00000000]
  -o, --openchain
          Enable to submit the matching signature to Openchain after a successful match
  -h, --help
          Print help

```
