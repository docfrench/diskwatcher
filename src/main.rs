use std::{process::Command};

#[allow(dead_code)]
#[derive(Debug)]
struct Disk {
    name: String,        // 
    device: String,      // "sdg" used as lookup; label
    disk_type: String,   // "Parity" becomes a label
    status: String,      // "DISK_OK" could be a label, or converted to a 1.0/0.0 metric
    temp: i32,           
    num_errors: i32,     
    num_reads: i32,
    num_writes: i32,
    size: i32,           
    smarthdd: Option<SmartDataHDD>,
    smartssd: Option<SmartDataSSD>,
}
#[allow(dead_code)]
#[derive(Debug, Default)]
struct ArrayState {
    md_state: String, //"STARTED"
    md_resync_action: String,
    md_resync_pos: Option<i64>,
    md_resync_size: Option<i64>,
    md_resync_corr: Option<i32>,
    sb_sync_errs: Option<i32>,
    md_num_disks: i16,
    md_num_disabled: i16,
    md_num_invalid: i16,
    md_num_missing: i16,
    fs_state: String,
}


#[allow(dead_code)]
#[derive(Debug, Default)]
struct SmartDataHDD {
    raw_read_error_rate: Option<i32>,
    throughput_performance: Option<i32>,
    spin_up_time: Option<i32>,
    start_stop_count: Option<i32>,
    reallocated_sector_ct: Option<i32>,
    seek_error_rate: Option<i32>,
    seek_time_performance: Option<i32>,
    power_on_hours: Option<i32>,
    spin_retry_count: Option<i32>,
    power_cycle_count: Option<i32>,
    helium_level: Option<i32>,
    power_off_retract_count: Option<i32>,
    load_cycle_count: Option<i32>,
    temperature_celsius: Option<i32>,
    reallocated_event_count: Option<i32>,
    current_pending_sector: Option<i32>,
    offline_uncorrectable: Option<i32>,
    udma_crc_error_count: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
struct SmartDataSSD {
    critical_warning: Option<i64>,
    temperature: Option<i32>,
    available_spare: Option<i32>,
    available_spare_thresh: Option<i32>,
    percentage_used: Option<i32>,
    data_units_read: Option<i32>,
    data_units_written: Option<i32>,
    host_read_commands: Option<i32>,
    host_write_commands: Option<i32>,
    controller_busy_time: Option<i32>,
    power_cycles: Option<i32>,
    power_on_hours: Option<i32>,
    unsafe_shutdowns: Option<i32>,
    media_data_int: Option<i32>,
    error_inf_log: Option<i32>,
    warning_comp_tt: Option<i32>,
    critical_comp_tt: Option<i32>,
    temp_sen_1: Option<i32>,
    temp_sen_2: Option<i32>,
}


fn main() {


    let emhttp = Command::new("cat")
        .args(["/var/local/emhttp/disks.ini"])
        .output()
        .unwrap();


    let var_file = Command::new("cat")
        .args(["/var/local/emhttp/var.ini"])
        .output()
        .unwrap();



    println!("Disk list populated.\nScanning array...");

    let array_data: ArrayState = array_scanner(&var_file);
    println!("{:#?}", array_data);
    println!("Gathering basic disk info...");
    let mut disk_collection = process_disks(&emhttp);
    

    println!("Gathering SMART disk info...");
    for disk in &mut disk_collection {
        let disk_device = &disk.device;
        let smart_hdd: SmartDataHDD;
        let smart_ssd: SmartDataSSD;
        if disk_device.contains("nvme") {
            smart_ssd = scrape_ssd(disk_device);
            disk.smartssd = Some(smart_ssd);
            println!("SSD: {} completed", disk_device);
            //println!("smart: {:#?}", disk.smartssd);
        } else {
            smart_hdd = scrape_hdd(disk_device);
            disk.smarthdd = Some(smart_hdd);
            println!("HDD: {} completed", disk_device);
            //println!("smart: {:#?}", disk.smarthdd);
        }
        
    }

    println!("{:#?}", disk_collection);
}

fn scrape_hdd(device: &str) -> SmartDataHDD {
    let device_path = format!("/dev/{}", device);
    let raw_output = Command::new("smartctl")
        .args(["-A", &device_path])
        .output()
        .unwrap();



    let mut smart = SmartDataHDD::default();


    let str_output = String::from_utf8_lossy(&raw_output.stdout);
    for line in str_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 10 {
            continue;
        }
        if parts[0] == "ID#" || parts[0] == "===" || parts[0] == "Vendor" || parts[0] == "ATTRIBUTE_NAME" {
            continue;
        }
        match parts[1].trim() {
                    "Raw_Read_Error_Rate" => smart.raw_read_error_rate = parts[9].parse::<i32>().ok(),
                    "Throughput_Performance" => smart.throughput_performance = parts[9].parse::<i32>().ok(),
                    "Spin_Up_Time" => smart.spin_up_time = parts[9].parse::<i32>().ok(),
                    "Start_Stop_Count" => smart.start_stop_count = parts[9].parse::<i32>().ok(),
                    "Reallocated_Sector_Ct" => smart.reallocated_sector_ct = parts[9].parse::<i32>().ok(),
                    "Seek_Error_Rate" => smart.seek_error_rate = parts[9].parse::<i32>().ok(),
                    "Seek_Time_Performance" => smart.seek_time_performance = parts[9].parse::<i32>().ok(),
                    "Power_On_Hours" => smart.power_on_hours = parts[9].parse::<i32>().ok(),
                    "Spin_Retry_Count" => smart.spin_retry_count = parts[9].parse::<i32>().ok(),
                    "Power_Cycle_Count" => smart.power_cycle_count = parts[9].parse::<i32>().ok(),
                    "Helium_Level" => smart.helium_level = parts[9].parse::<i32>().ok(),
                    "Power-Off_Retract_Count" => smart.power_off_retract_count = parts[9].parse::<i32>().ok(),
                    "Load_Cycle_Count" => smart.load_cycle_count = parts[9].parse::<i32>().ok(),
                    "Temperature_Celsius" => smart.temperature_celsius = parts[9].parse::<i32>().ok(),
                    "Reallocated_Event_Count" => smart.reallocated_event_count = parts[9].parse::<i32>().ok(),
                    "Current_Pending_Sector" => smart.current_pending_sector = parts[9].parse::<i32>().ok(),
                    "Offline_Uncorrectable" => smart.offline_uncorrectable = parts[9].parse::<i32>().ok(),
                    "UDMA_CRC_Error_Count" => smart.udma_crc_error_count = parts[9].parse::<i32>().ok(),
                  
                    _ => {}
                }


    }

    smart
}


fn scrape_ssd(device: &str) -> SmartDataSSD {
    let device_path = format!("/dev/{}", device);
    let raw_output = Command::new("smartctl")
        .args(["-A", &device_path])
        .output()
        .unwrap();

    let mut smart = SmartDataSSD::default();

    let str_output = String::from_utf8_lossy(&raw_output.stdout);
    for line in str_output.lines() {
        //let cleaned_line = line.trim().replace('%', "");
             
        
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let label = parts[0].trim();
        let value_with_unit_t = parts[1].trim().replace("%", "").replace(",", "");
        let value_parts: Vec<&str> = value_with_unit_t.split_whitespace().collect();
        if value_parts.is_empty() {
            continue;
        }
        let value_with_unit = value_parts[0];

        match label {
                    "Critical Warning" => smart.critical_warning = value_with_unit.parse::<i64>().ok(),
                    "Temperature" => smart.temperature = value_with_unit.parse::<i32>().ok(),
                    "Available Spare" => smart.available_spare = value_with_unit.parse::<i32>().ok(),
                    "Available Spare Threshold" => smart.available_spare_thresh = value_with_unit.parse::<i32>().ok(),
                    "Percentage Used" => smart.percentage_used = value_with_unit.parse::<i32>().ok(),
                    "Data Units Read" => smart.data_units_read = value_with_unit.parse::<i32>().ok(),
                    "Data Units Written" => smart.data_units_written = value_with_unit.parse::<i32>().ok(),
                    "Host Read Commands" => smart.host_read_commands = value_with_unit.parse::<i32>().ok(),
                    "Host Write Commands" => smart.host_write_commands = value_with_unit.parse::<i32>().ok(),
                    "Controller Busy Time" => smart.controller_busy_time = value_with_unit.parse::<i32>().ok(),
                    "Power Cycles" => smart.power_cycles = value_with_unit.parse::<i32>().ok(),
                    "Power On Hours" => smart.power_on_hours = value_with_unit.parse::<i32>().ok(),
                    "Unsafe Shutdowns" => smart.unsafe_shutdowns = value_with_unit.parse::<i32>().ok(),
                    "Media and Data Integrity Errors" => smart.media_data_int = value_with_unit.parse::<i32>().ok(),
                    "Error Information Log Entries" => smart.error_inf_log = value_with_unit.parse::<i32>().ok(),
                    "Warning Comp. Temperature Time" => smart.warning_comp_tt = value_with_unit.parse::<i32>().ok(),
                    "Critical Comp. Temperature Time" => smart.critical_comp_tt = value_with_unit.parse::<i32>().ok(),
                    "Temperature Sensor 1" => smart.temp_sen_1 = value_with_unit.parse::<i32>().ok(),
                    "Temperature Sensor 2" => smart.temp_sen_2 = value_with_unit.parse::<i32>().ok(),
                    _ => {}
                }
        };


    smart
}

fn process_disks(emhttp: &std::process::Output) -> Vec<Disk> {
    let str_output = String::from_utf8_lossy(&emhttp.stdout);
    let mut disk_info: Vec<Disk> = Vec::new();

    let mut d_name = String::new();
    let mut d_device = String::new();
    let mut d_type = String::new();
    let mut d_status = String::new();
    let mut d_temp = 0;
    let mut d_num_errors = 0;
    let mut d_num_reads = 0;
    let mut d_num_writes = 0;
    let mut d_size = 0;
    let mut has_data = false; // tracks whether we've accumulated a real disk yet

    for line in str_output.lines() {
        let trimmed = line.trim();

        // Boundary check happens FIRST, independent of the "=" check
        if trimmed.starts_with("[") {
            if has_data && !d_device.is_empty() {
                disk_info.push(Disk {
                    name: d_name.clone(),
                    device: d_device.clone(),
                    disk_type: d_type.clone(),
                    status: d_status.clone(),
                    temp: d_temp,
                    num_errors: d_num_errors,
                    num_reads: d_num_reads,
                    num_writes: d_num_writes,
                    size: d_size,
                    smarthdd: None,
                    smartssd: None,
                });
            }
            // reset for the next disk
            d_name.clear();
            d_device.clear();
            d_type.clear();
            d_status.clear();
            d_temp = 0;
            d_num_errors = 0;
            d_num_reads = 0;
            d_num_writes = 0;
            d_size = 0;
            
            has_data = true;
            continue; // nothing else to do with a header line
        }

        if trimmed.contains("=") {
            let parts: Vec<&str> = trimmed.split("=").collect();
            if parts.len() >= 2 {
                let value = parts[1].trim_matches('"');
                match parts[0].trim() {
                    "name" => d_name = value.to_string(),
                    "device" => d_device = value.to_string(),
                    "type" => d_type = value.to_string(),
                    "status" => d_status = value.to_string(),
                    "temp" => d_temp = value.parse().unwrap_or(0),
                    "numErrors" => d_num_errors = value.parse().unwrap_or(0),
                    "numReads" => d_num_reads = value.parse().unwrap_or(0),
                    "numWrites" => d_num_writes = value.parse().unwrap_or(0),
                    "size" => d_size = value.parse().unwrap_or(0),
                    _ => {}
                }

            }
        }
    }

    // the last disk in the file never hits another "[" header, so push it here
    if has_data && !d_device.is_empty() {
        disk_info.push(Disk {
            name: d_name,
            device: d_device,
            disk_type: d_type,
            status: d_status,
            temp: d_temp,
            num_errors: d_num_errors,
            num_reads: d_num_reads,
            num_writes: d_num_writes,
            size: d_size,
            smarthdd: None,
            smartssd: None,
        });
    }

    disk_info
}


fn array_scanner(var_file: &std::process::Output) -> ArrayState {
    let str_output = String::from_utf8_lossy(&var_file.stdout);


    let mut md_state= String::new(); //"STARTED"
    let mut md_resync_action= String::new();
    let mut md_resync_pos = None;
    let mut md_resync_size = None;
    let mut md_resync_corr = None;
    let mut sb_sync_errs = None;
    let mut md_num_disks = 0;
    let mut md_num_disabled = 0;
    let mut md_num_invalid = 0;
    let mut md_num_missing = 0;
    let mut fs_state= String::new();


    for line in str_output.lines() {
        let trimmed = line.trim();

        if trimmed.contains("=") {
            let parts: Vec<&str> = trimmed.split("=").collect();
            if parts.len() >= 2 {
                let value = parts[1].trim_matches('"');
                match parts[0].trim() {
                    "mdState" => md_state = value.to_string(),
                    "mdResyncAction" => md_resync_action = value.to_string(),
                    "mdResyncPos" => md_resync_pos = value.parse::<i64>().ok(),
                    "mdResyncSize" => md_resync_size = value.parse::<i64>().ok(),
                    "mdResyncCorr" => md_resync_corr = value.parse::<i32>().ok(),
                    "sbSyncErrs" => sb_sync_errs = value.parse::<i32>().ok(),
                    "mdNumDisks" => md_num_disks = value.parse::<i16>().unwrap_or(0),
                    "mdNumDisabled" => md_num_disabled = value.parse::<i16>().unwrap_or(0),
                    "mdNumInvalid" => md_num_invalid = value.parse::<i16>().unwrap_or(0),
                    "mdNumMissing" => md_num_missing = value.parse::<i16>().unwrap_or(0),
                    "fsState" => fs_state = value.to_string(),

                    _ => {}
                }

            }
        }
    

    }
    ArrayState {
        md_state,
        md_resync_action,
        md_resync_pos,
        md_resync_size,
        md_resync_corr,
        sb_sync_errs,
        md_num_disks,
        md_num_disabled,
        md_num_invalid,
        md_num_missing,
        fs_state,
    }
    
}
