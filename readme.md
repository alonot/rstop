# RSTOP

A tool to view your storage with percent analysis, directly inside your terminal.
- This app provides you a feature called *Aggregator*.<br/>
    *Aggregator* : As name suggests this aggregates all the files with same extension in the directory and show total space these files with same extension consumes. One can expand these aggregators to look at the stats of individual files also.
- Also Stats the folder or file and display various infos like modified date, created date, etc..
- Provides keyboard functionallity to move between different components of the screen.

That's all. Below is the specification about how the app was build

NOTE: if running from root or from a directory whose read permission is not to with current user. then run using **sudo** 
`
    sudo ./build/rstop <Optional<folder_location>>
`

**WORK IN PROGRESS**

---

# Frontend

- Using `cncurses` Component Library for `ncurses`.

# Backend

* To prevent too much contention on same memory location backend will send back data by cloning it but in packets.

* Work Queue
* On CONN_CLOSE, All the works of that Connedtion is closed 

Problems:
1. How will frontend run an infinite loop ?
    Solution: we will use handler technic.. 
    A separate thread will run and check for message from backend, when message recieved it will call this global handler set when App{} is called. This handle will then call etState() to update the frontend accordingly. 

2. If backend send data in packets evry second, then frontend may overload on bigger rendering. Solution frontend will send acknowledge that update is finished, then only backend will start next 1 second timer.