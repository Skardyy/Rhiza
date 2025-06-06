<div align="center">
  
<img src="https://github.com/user-attachments/assets/19a6ec4f-05ae-4655-8973-1beddb59e36b" width="256"/>

# Rhiza
![Downloads](https://img.shields.io/crates/d/rhiza?style=for-the-badge) ![Version](https://img.shields.io/crates/v/rhiza?style=for-the-badge)  

Rhiza is a simple and
easy-to-use tool to create shortcuts and add entries to the path.

from crawling with strict filtering, to hands down system-wide search for a name you want to find. don't let those apps hide from you anymore!

***Rhiza is for windows only***

</div>

## Installation🔧
<details>
<summary>via cargo</summary>

```sh
cargo install rhiza
```
</details>
<details>
<summary>via winget</summary>

```sh
winget install skardyy.rhiza
```
</details>
<details>
<summary>via installer</summary>

> install and run the .msi installer from [here](https://github.com/Skardyy/rhiza/releases/latest)
</details>

## Usage💡
### Crawl
```sh
rhz crawl
```
https://github.com/user-attachments/assets/c0f0d8f3-4d8f-4629-928b-7e811458a90a

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
https://github.com/user-attachments/assets/e3010698-b8d9-49d6-b820-4c173e914a4f

### Path  
same as add ~ just for adding into path  
```sh
rhz path NAME
```  
https://github.com/user-attachments/assets/09e8ebe3-89b9-4ee0-b908-40265935518b

> [!Note]  
> For both the **Path** and **Add** functions  
> if the user have **fzf** installed in the machine and he didn't specify a search term  
> fzf will open to search for the recommended files

https://github.com/user-attachments/assets/4014db4b-90d5-4910-a7f0-df3235c18045

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
