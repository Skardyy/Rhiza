<h1 align="center">Rhiza</h1>
<p align="center">A blazingly fast app linker</p>
<div align="center">

[![Static Badge](https://img.shields.io/badge/crates.io-1e2029?style=flat&logo=rust&logoColor=f74b00&label=find%20at&labelColor=15161b)](https://crates.io/crates/rhiza)
</div>

---

> [!Note]
> Rhiza is for windows only  
> linux has better options already

### Table of Content  
* [Installation](#Installation)
* [Usage](#Usage)
  * [Crawl](#Crawl)
  * [Add](#Add)
  * [Path](#Path)
  * [View](#View)
  * [Edit](#Edit)
  * [Run](#Run)

## Installation🔧
```sh
cargo install rhiza
```

## Usage💡
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

> * **Crawl** is mostly to find gui apps and games
> * there are more filtering and logic to prompt you only for relevant apps to link
> * you will be prompted for new apps you didn't link to before (apps you said no before won't be prompted again)
  
  
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


### Path  
same as add ~ just for adding into path  
```sh
rhz path NAME
```  

https://github.com/user-attachments/assets/63edb5d9-ffa0-4b21-84eb-105db17346db


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
> Rhiza means "Root" in greek 🌱🌿  
> rooting those apps for you  
