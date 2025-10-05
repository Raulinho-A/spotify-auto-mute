use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::{System};

use windows::core::{Result as WinResult};
use windows::core::ComInterface;
use windows::Win32::Media::Audio::{IMMDevice, ISimpleAudioVolume};
use windows::Win32::System::Com::CoCreateInstance;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    System::Com::{CoInitializeEx, COINIT_MULTITHREADED, CoUninitialize},
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    },
    Media::Audio::{
        eMultimedia, eRender, IAudioSessionControl, IAudioSessionControl2,
        IAudioSessionEnumerator, IAudioSessionManager2, IMMDeviceEnumerator,
        MMDeviceEnumerator,
    },
    System::Com::CLSCTX_ALL,
};

const ADVERTISEMENT: &str = "Advertisement";

fn com_init() -> WinResult<()> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED)?; }
    Ok(())
}

fn com_uninit() {
    unsafe { CoUninitialize(); }
}

struct ComGuard;
impl Drop for ComGuard {
    fn drop(&mut self) {
        com_uninit();
    }
}

fn find_spotify_pid() -> Option<HashMap<String, Vec<u32>>> {
    // let mut sys = System::new_all(); // todo sys
    // sys.refresh_all(); //

    let mut sys = System::new();
    sys.refresh_processes();
    
    let mut pids_map = HashMap::new();

    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_ascii_lowercase();

        if name.contains("spotify") {
            pids_map.entry(proc_.name().to_string()).or_insert_with(Vec::new).push(proc_.pid().as_u32());
        }
    }

    if pids_map.is_empty() {
        None
    } else {
        Some(pids_map)
    }
}

#[repr(C)]
struct EnumData {
    target_pid: u32,
    found_hwnd: HWND,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);

    if IsWindowVisible(hwnd).as_bool() {
        let mut pid: u32 = 0;

        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == data.target_pid {
            data.found_hwnd = hwnd;
            return BOOL(0);
        }
    }

    BOOL(1)
}

fn find_main_window_of_pid(pid: u32) -> Option<HWND> {
    let mut data = EnumData {
        target_pid: pid,
        found_hwnd: HWND(0)
    };
    let lparam = LPARAM(&mut data as *mut EnumData as isize);
    let _ = unsafe { EnumWindows(Some(enum_windows_proc), lparam) };
    if data.found_hwnd.0 != 0 { Some(data.found_hwnd) } else { None }
}

fn window_title(hwnd: HWND) -> Option<String> {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
    if len == 0 { return None; }
    Some(String::from_utf16_lossy(&buf[..len]))
}

fn set_mute_for_pid(pid_target: u32, mute: bool) -> WinResult<()> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

        let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;

        let sessions: IAudioSessionEnumerator = manager.GetSessionEnumerator()?;
        let count = sessions.GetCount()?;

        for i in 0..count {
            let control: IAudioSessionControl = sessions.GetSession(i)?;

            if let Ok(ctrl2) = control.cast::<IAudioSessionControl2>() {
                let pid = ctrl2.GetProcessId()?;
                if pid == pid_target {
                    if let Ok(vol) = control.cast::<ISimpleAudioVolume>() {
                        let current_mute = vol.GetMute()?;
                        let is_muted = current_mute.as_bool();

                        if is_muted != mute {
                            vol.SetMute(mute, std::ptr::null())?;
                            println!(
                                "{}",
                                if mute {
                                    "\x1b[31m🔇 Anuncio detectado\x1b[0m → muteando Spotify..."
                                } else {
                                    "\x1b[32m🎵 Música normal\x1b[0m → desmuteando..."
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// fn main() -> WinResult<()> {
//     let _com = ComGuard;
//     com_init()?;
//     match find_spotify_pid() {
//         Some(processes) => {
//             println!("Spotify encontrado! PIDs");
//             for (name, pids) in processes {
//                 println!("   • {} → {:?}", name, pids)
//             }
//         },
//         None => println!("Spotify no está corriendo."),
//     }
//     Ok(())
// }

// fn main() -> WinResult<()> {
//     let _com = ComGuard;
//     com_init()?;

//     match find_spotify_pid() {
//         Some(processes) => {
//             println!("Spotify encontrado! PIDs y ventanas:");
//             for (name, pids) in processes {
//                 println!("   • {} → {:?}", name, pids);

//                 for pid in pids {
//                     if let Some(hwnd) = find_main_window_of_pid(pid) {
//                         println!("           Ventana encontrada para PID {} >>> {:?}", pid, hwnd);
//                     } else {
//                         println!("           No se encontró ventana para PID {}", pid);
//                     }
//                 }
//             }
//         },
//         None => println!("Spotify no está corriendo."),
//     }

//     Ok(())
// }

// fn main() -> WinResult<()> {
//     let _com = ComGuard;
//     com_init()?;

//     match find_spotify_pid() {
//         Some(processes) => {
//             println!("Spotify encontrado! PIDs y ventanas:");
//             for (name, pids) in processes {
//                 println!("   • {} → {:?}", name, pids);

//                 for pid in pids {
//                     if let Some(hwnd) = find_main_window_of_pid(pid) {
//                         println!("           Ventana encontrada para PID {} >>> {:?}", pid, hwnd);
//                         if let Some(title) = window_title(hwnd) {
//                             println!("              Ventana: {:?} → \"{}\"", hwnd, title);
//                             if title.contains(ADVERTISEMENT) {
//                                 println!("                  ¡Encontró ! {}", title);
//                             }
//                         }
//                     } else {
//                         println!("           No se encontró ventana para PID {}", pid);
//                     }
//                 }
//             }
//         },
//         None => println!("Spotify no está corriendo."),
//     }

//     Ok(())
// }


// fn main() -> WinResult<()> {
//     let _guard = ComGuard;
//     com_init()?;

//     println!(" spotify-auto-mute → ON (Windows)");

//     match find_spotify_pid() {
//         Some(processes) => {
//             println!("Spotify encontrado! PIDs y ventanas:");
//             for (name, pids) in processes {
//                 println!("   • {} → {:?}", name, pids);

//                 for pid in pids {
//                     if let Some(hwnd) = find_main_window_of_pid(pid) {
//                         if let Some(title) = window_title(hwnd) {
//                             println!("              Ventana: {:?} → \"{}\"", hwnd, title);

//                             if title.contains(ADVERTISEMENT) {
//                                 println!(" ¡Anuncio detectado! Silenciando Spotify...");
//                                 set_mute_for_pid(pid, true)?;
//                             } else {
//                                 println!(" Reproducción normal. Desmuteando...");
//                                 set_mute_for_pid(pid, false)?;
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         None => println!("Spotify no está corriendo."),
//     }

//     Ok(())
// }

fn main() -> WinResult<()> {
    let _guard = ComGuard;
    com_init()?;
    print!("\x1B[2J\x1B[1;1H");
    println!(" spotify-auto-mute → ON (Windows)");

    let mut cached_pid: Option<u32> = None;

    let mut sleep_duration = Duration::from_secs(2);

    let mut last_title = String::new();

    let mut in_advert = false;

    loop {
        // Buscar Spotify solo si no tenemos PID válido
        if cached_pid.is_none() {
            cached_pid = find_spotify_pid()
                .and_then(|map| map.get("Spotify.exe").cloned())
                .and_then(|v| {
                    v.into_iter()
                        .find(|pid| find_main_window_of_pid(*pid).is_some())
                });
            sleep_duration = Duration::from_secs(30);
        } else if !in_advert {
            sleep_duration = Duration::from_secs(2);
        }

        // Si tenemos un PID cacheado, procedemos
        if let Some(pid) = cached_pid {
            if let Some(hwnd) = find_main_window_of_pid(pid) {
                if let Some(title) = window_title(hwnd) {
                    // Solo entra a ejecutar set_mute_for_pid si el título cambió
                    if title != last_title {
                        println!(" Ventana activa: \"{}\"", title);

                        if title.contains(ADVERTISEMENT) {
                            set_mute_for_pid(pid, true)?;
                            in_advert = true;
                            sleep_duration = Duration::from_millis(300);
                        } else {
                            set_mute_for_pid(pid, false)?;
                            in_advert = false;
                            sleep_duration = Duration::from_secs(2);
                        }

                        last_title = title.clone();
                    }
                }
            } else {
                println!("Ventana cerrada. Spotify pudo cerrarse...");
                cached_pid = None;
                last_title.clear();
                in_advert = false;
            }
        } else {
            println!("Spotify no está corriendo. Esperando...");
        }

        // println!("Esperando {} ms...", sleep_duration.as_millis()); // debug sleep_duration
        thread::sleep(sleep_duration);

    }
}

// // Ejercicio 1
// use sysinfo::{System};

fn list_spotify_exe_processes() {
    let mut sys = System::new_all();
    sys.refresh_all();

    for (pid, proc_) in sys.processes() {
        let name = proc_.name().to_ascii_lowercase();
        if name.contains("spotify") {
            println!("{} -> {}", pid, proc_.name());
        }
    }
}

// // Ejercicio 2
// use std::{ffi::c_void, ptr, time::Duration, thread};

// fn manejando_hilos() {
//     println!("Hola bajo nivel de Rust \n");

//     let raw_ptr: *mut c_void = ptr::null_mut();
//     println!("Puntero vacio (tipo void* en C): {:?}", raw_ptr);

//     let numero: i32 = 42;

//     let puntero: *const i32 = &numero;
//     println!("La direccion en memoria de `numero` es: {:?}", puntero);

//     unsafe {
//         println!("El valor al que apunta: {}", *puntero);
//     }

//     let handle = thread::spawn(|| {
//         for i in 1..=7 {
//             println!("Hilo secundario: paso {i}");
//             thread::sleep(Duration::from_millis(500));
//         }
//         "Hilo terminado"
//     });

//     thread::sleep(Duration::from_secs(6));
//     println!("Hilo principal esperando");

//     let resultado = handle.join().unwrap();
//     println!("{resultado}");

//     println!("fin del programa");
// }