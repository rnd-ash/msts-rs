# MSTS-RS
Rust based port of Microsoft Train Simulator 2001 (MSTS)

This repository contains all the code for the custom port of MSTS.

## Assets

Since MSTS is now abandonware, you can either download the assets directly from [here](https://drive.google.com/file/d/1Lj3_5K-JlfXYHSsRNNCjZm1FdKyROYEj/view?usp=sharing), or setup a new installation using the ISOs from [here](https://www.jumpjet.info/Classic-Games/Windows/MSTS/msts.htm), and copy the folders specified below.

### Copying folders
You will need to place the following folders from your original MSTS installation here, or from the download link:

* ROUTES
* TRAINS
* SOUND
* GLOBAL
* GUI
* FONTS

Thats it! Just compile and run the application from this directory with 
```
cargo run --release
```
