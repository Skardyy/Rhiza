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
cargo install --path .
```

# Usage💡
### Crawl
```sh
rhz crawl
```
https://github.com/user-attachments/assets/61b4cee4-70f5-4f0d-9e24-a7b06efacd4a

to find potential apps to link (walks recursively)
defaults:
* ~\Desktop
* ~\AppData\Roaming\Microsoft\Windows\Start Menu
* C:\ProgramData\Microsoft\Windows\Start Menu
there are more filtering and logic to prompt the user only for relevant apps to link
  
you will be prompted for new apps you didn't link to before (apps you said no before won't be prompted again)
  
you can also
```sh
rhz crawl "/PATH/TO/DIR"
```

### Add
you can search for a single app across the entire file-system (ignores hidden folders and Windows/Microsoft ones)
```sh
rhz add NAME
```
https://github.com/user-attachments/assets/8fad0bf8-0390-4471-a5c4-39f9d0c22117  

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
it will open the ~/.rhiza/ folder where you can:  
* edit the config to not automatically re add the deleted items  
* delete the url/lnk file from the src dir (after doing `rhz run` the shell and menu links will be removed as well)

### Run
finally you can create the lnk files using
```
rhz run
```
https://github.com/user-attachments/assets/d3e529c3-fbc7-45dd-80f8-341c012fecaa

it will create the bin and src files and allow you to use your shortcuts in the shell and in the widnows menu! (`⊞ Win`)
  
> [!Tip]
> did you know?  
> Rhiza is the spirit of roots and growth 🌱🌿  
> rooting those apps for you  
