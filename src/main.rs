use std::thread::sleep;

use sysinfo::{ComponentExt, DiskExt, NetworkExt, System, SystemExt};

use std::io::{stdout, Write};

mod tgph_format;
use tgph_format::TGPH;

fn main() {
    let mut points_saved = 0;
    let mut sys = System::new_all();
    let mut stdout = stdout();

    let mut tgph = TGPH::default();
    loop {
        sys.refresh_all();

        for disk in sys.disks() {
            tgph.append(
                (disk.total_space() / 1024 / 1024) as u32,
                "disk_total_space",
            );
            tgph.append(
                (disk.available_space() / 1024 / 1024) as u32,
                "disk_available_space",
            );
            tgph.append(disk.name().to_str().unwrap().to_string(), "disk_name");
        }

        for (interface_name, data) in sys.networks() {
            tgph.append(interface_name.to_string(), "iface_name");
            tgph.append(data.received() as u32, "iface_received");
            tgph.append(data.transmitted() as u32, "iface_transmitted");
        }

        for component in sys.components() {
            tgph.append(component.label().to_string(), "thermal_component_name");
            tgph.append(component.temperature(), "thermal_component_temp");
        }

        tgph.append(sys.cpus().len() as u32, "cpu_count");

        tgph.append((sys.total_memory() / 1024) as u32, "total_memory_kb");
        tgph.append((sys.used_memory() / 1024) as u32, "used_memory_kb");
        tgph.append((sys.total_swap() / 1024) as u32, "total_swap_kb");
        tgph.append((sys.used_swap() / 1024) as u32, "used_swap_kb");

        tgph.append(
            sys.kernel_version()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "kernel_version",
        );
        tgph.append(
            sys.os_version()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "os_version",
        );
        tgph.append(
            sys.host_name()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "host_name",
        );

        let mut output_file = std::fs::File::create("data.tgph").unwrap();
        tgph.serialize_into(&mut output_file).unwrap();

        points_saved += 1;

        print!("\rSaved {points_saved} snapshots");
        stdout.flush().unwrap();

        sleep(std::time::Duration::from_secs(15));
    }
}
