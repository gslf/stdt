```
          __      .___ __   
  _______/  |_  __| _//  |_ 
 /  ___/\   __\/ __ |\   __\
 \___ \  |  | / /_/ | |  |  
/____  > |__| \____ | |__|  
     \/            \/       
```
Minimal, zero-dependency standard tools for Rust.|

[READ THE DOCS](https://docs.rs/stdt/latest/stdt/)

## Available Tools
### 🛠️ stdt::utils
A lightweight collection of everyday coding utilities. Simple, dependency-free helpers to make common tasks faster and cleaner. These are the available functions:
- **stdt::utils::clear_cli** Console clearing with raw ANSI escape sequences. [Read the docs.]()
- **stdt::utils::type_of** Find out what a value’s type is. [Read the docs.]()
- **stdt::utils::devops** Load enviroment variables from an .env file. [Read the docs.]()

### 📄 stdt::json
A minimal yet complete implementation of JSON handling in. It defines a Value type that represents any JSON data and supports convenient conversions from native Rust types. A lightweight recursive descent parser turns JSON text into a Value while providing detailed error reporting, and a serializer implements Display to produce valid JSON strings with proper escaping and formatting.

[Read the docs.]()


## 🎯 Philosophy
- Zero dependencies
- Self-explanatory code with inline docs
- Small, composable building blocks

## 📦 Installation
Add to your Cargo.toml, or copy/paste individual files from utils/ if you want ultimate minimalism.

```toml
[dependencies]
stdt = "0.0.4"
```


#### :#/ GSLF - https://gslf.it
