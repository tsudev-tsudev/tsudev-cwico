// Release Windows builds must not open a console window behind the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cwico_app_lib::run()
}
