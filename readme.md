# Rhiza: a blazingly fast app linker 🚀
only made for windows (linux has way better options already)
### Table of Content  
* [Requirements](#Requirements)
* [Installation](#Installation)
* [Usage](#Usage)
  * [Crawl](#Crawl)
  * [Add](#Add)
  * [View](#View)
  * [Edit](#Edit)
  * [Run](#Run)

# Requirements📝
* [rust](https://www.rust-lang.org/)

# Installation🔧
```sh
git clone https://github.com/Skardyy/Rhiza
cd rhiza
cargo build --release
./target/release/rhz install
```

# Usage💡
### Crawl
```sh
rhz crawl
```
to find potential apps to link  
defaults to (recursive):
* ~\Desktop
* ~\AppData\Roaming\Microsoft\Windows\Start Menu
* C:\ProgramData\Microsoft\Windows\Start Menu
  
you will be prompted for new apps you didn't link to before
  
you can also
```sh
rhz crawl -p "/path/to/dir"
```
and to crawl recursively
```sh
rhz crawl -p "path/to/dir" -r
```

### Add
you can add apps to link manually (why tho)
```
rhz add -p "path/to/app"
```
once again you will be prompted if the app is not linked already

### View
you can view all linked apps and their config
```
rhz view
```
it will print it in a formatted json

### Edit
or maybe you want to edit the config
```
rhz edit
```
it will open the config in your preferred editor

### Run
finally you can create the lnk files using
```
rhz run
```
it will create the lnk files and allow you to use your shortcuts in shell and in the quick access menu
  
> \[!Tip]
> did you know?  
> Rhiza is the spirit of roots and growth, embodying the hidden strength and connection of the earth. 🌱🌿
