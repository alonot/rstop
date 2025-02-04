TODO:

    1. Decide on how to display files and folders. Maybe with "-name" keeping the actual aggregator there only, on double click
        - To display, Increase the height of the aggregator **StorageWin** to accomodate all those then add as much the StorageWin as its child, keep a flag in aggregator "expanded". 
        The first child now will be actually a storage win and then another win will contain all other *StorageWin* s.
        Structure may be now: 
            StorageWin(Aggregator)
                - Header: shows aggregator.
                    - Name | Size | percent | <sort by name> | <sort by Size>
                - Win: Shows each file.
                    - StorageWin
                    - StorageWin
                    - StorageWin
                    ....
    [Monday]

    2. Scroll [Tuesday]
    3. Decide upon what to display when clicked on Aggregator in storage window.
    4. Multiple Directories stored together [Wednesday]
    5. Sort button (by size, name) [Thursday]
    6. Colors, Background [Friday]
    7. Reload button [Saturday]
    8. Product Release [Saturday]
