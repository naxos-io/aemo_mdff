#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

pub mod TRANSACTION_CODE {
    pub const A: &'static str = "Alteration";
    pub const C: &'static str = "Meter Reconfiguration";
    pub const G: &'static str = "Re-energisation";
    pub const D: &'static str = "De-energisation";
    pub const E: &'static str = "Forward Estimate";
    pub const N: &'static str = "Normal Read";
    pub const O: &'static str = "Other";
    pub const S: &'static str = "Special Read";
    pub const R: &'static str = "Removal of Meter";
}

pub mod UOM {
    pub const MWh: &'static UomMeta   = &UomMeta { name: "Megawatt Hour", multiplier: 1e6 as f64 };
    pub const kWh: &'static UomMeta   = &UomMeta { name: "Kilowatt Hour", multiplier: 1e3 as f64 };
    pub const Wh: &'static UomMeta    = &UomMeta { name: "Watt Hour", multiplier: 1 as f64 };
    pub const MW: &'static UomMeta    = &UomMeta { name: "Megawatt", multiplier: 1e6 as f64 };
    pub const kW: &'static UomMeta    = &UomMeta { name: "Kilowatt", multiplier: 1e3 as f64 };
    pub const W: &'static UomMeta     = &UomMeta { name: "Watt", multiplier: 1 as f64 };
    pub const MVArh: &'static UomMeta = &UomMeta { name: "Megavolt Ampere Reactive Hour", multiplier: 1e6 as f64 };
    pub const kVArh: &'static UomMeta = &UomMeta { name: "Kilovolt Ampere Reactive Hour", multiplier: 1e3 as f64 };
    pub const VArh: &'static UomMeta  = &UomMeta { name: "Volt Ampere Reactive Hour", multiplier: 1 as f64 };
    pub const MVAr: &'static UomMeta  = &UomMeta { name: "Megavolt Ampere Reactive", multiplier: 1e6 as f64 };
    pub const kVAr: &'static UomMeta  = &UomMeta { name: "Kilovolt Ampere Reactive", multiplier: 1e3 as f64 };
    pub const VAr: &'static UomMeta   = &UomMeta { name: "Volt Ampere Reactive", multiplier: 1 as f64 };
    pub const MVAh: &'static UomMeta  = &UomMeta { name: "Megavolt Ampere Hour", multiplier: 1e6 as f64 };
    pub const kVAh: &'static UomMeta  = &UomMeta { name: "Kilovolt Ampere Hour", multiplier: 1e3 as f64 };
    pub const VAh: &'static UomMeta   = &UomMeta { name: "Volt Ampere Hour", multiplier: 1 as f64 };
    pub const MVA: &'static UomMeta   = &UomMeta { name: "Megavolt Ampere", multiplier: 1e6 as f64 };
    pub const kVA: &'static UomMeta   = &UomMeta { name: "Kilovolt Ampere", multiplier: 1e3 as f64 };
    pub const VA: &'static UomMeta    = &UomMeta { name: "Volt Ampere", multiplier: 1 as f64 };
    pub const kV: &'static UomMeta    = &UomMeta { name: "Kilovolt", multiplier: 1e3 as f64 };
    pub const V: &'static UomMeta     = &UomMeta { name: "Volt", multiplier: 1 as f64 };
    pub const kA: &'static UomMeta    = &UomMeta { name: "Kiloampere", multiplier: 1e3 as f64 };
    pub const A: &'static UomMeta     = &UomMeta { name: "Ampere", multiplier: 1 as f64 };
    pub const pf: &'static UomMeta    = &UomMeta { name: "Power Factor", multiplier: 1 as f64 };

    pub struct UomMeta {
        name: &'static str,
        multiplier: f64
    }
}

pub mod QUALITY {
    pub const A: &'static str = "Actual Data";
    pub const E: &'static str = "Forward Estimated Data";
    pub const F: &'static str = "Final Substituted Data";
    pub const N: &'static str = "Null Data";
    pub const S: &'static str = "Substituted Data";
    pub const V: &'static str = "Variable Data";
}

pub mod METHOD {
    pub const FLAG_11: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Check", description: "" };
    pub const FLAG_12: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Calculated", description: "" };
    pub const FLAG_13: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "SCADA", description: "" };
    pub const FLAG_14: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Like Day", description: "" };
    pub const FLAG_15: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Average Like Day", description: "" };
    pub const FLAG_16: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Agreed", description: "" };
    pub const FLAG_17: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Linear", description: "" };
    pub const FLAG_18: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Alternate", description: "" };
    pub const FLAG_19: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::Arr(&[&1, &2, &3, &4]), short_descriptor: "Zero", description: "" };
    pub const FLAG_51: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Previous Year", description: "" };
    pub const FLAG_52: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Previous Read", description: "" };
    pub const FLAG_53: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Revision", description: "" };
    pub const FLAG_54: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Linear", description: "" };
    pub const FLAG_55: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Agreed", description: "" };
    pub const FLAG_56: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Prior to First Read - Agreed", description: "" };
    pub const FLAG_57: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Customer Class", description: "" };
    pub const FLAG_58: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(5), short_descriptor: "Zero", description: "" };
    pub const FLAG_61: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Previous Year", description: "" };
    pub const FLAG_62: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Previous Read", description: "" };
    pub const FLAG_63: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Customer Class", description: "" };
    pub const FLAG_64: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Agreed", description: "" };
    pub const FLAG_65: &'static MethodMeta = &MethodMeta { typ: &["EST"], installation_type: OneOrArr::One(6), short_descriptor: "ADL", description: "" };
    pub const FLAG_66: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Revision", description: "" };
    pub const FLAG_67: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Customer Read", description: "" };
    pub const FLAG_68: &'static MethodMeta = &MethodMeta { typ: &["EST", "SUB"], installation_type: OneOrArr::One(6), short_descriptor: "Zero", description: "" };
    pub const FLAG_71: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(7), short_descriptor: "Recalculation", description: "" };
    pub const FLAG_72: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(7), short_descriptor: "Revised Table", description: "" };
    pub const FLAG_73: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(7), short_descriptor: "Revised Algorithm", description: "" };
    pub const FLAG_74: &'static MethodMeta = &MethodMeta { typ: &["SUB"], installation_type: OneOrArr::One(7), short_descriptor: "Agreed", description: "" };
    pub const FLAG_75: &'static MethodMeta = &MethodMeta { typ: &["EST"], installation_type: OneOrArr::One(7), short_descriptor: "Existing Table", description: "" };

    pub struct MethodMeta {
        typ: &'static [&'static str],
        installation_type: OneOrArr,
        short_descriptor: &'static str,
        description: &'static str,
    }

    enum OneOrArr {
        One(u8),
        Arr(&'static [&'static u8])
    }
}

pub mod REASON {
    pub const CODE_0: &'static str = "Free Text Description";
    pub const CODE_1: &'static str = "Meter/Equipment Changed";
    pub const CODE_2: &'static str = "Extreme Weather/Wet";
    pub const CODE_3: &'static str = "Quarantine";
    pub const CODE_4: &'static str = "Savage Dog";
    pub const CODE_5: &'static str = "Meter/Equipment Changed";
    pub const CODE_6: &'static str = "Extreme Weather/Wet";
    pub const CODE_7: &'static str = "Unable To Locate Meter";
    pub const CODE_8: &'static str = "Vacant Premise";
    pub const CODE_9: &'static str = "Meter/Equipment Changed";
    pub const CODE_10: &'static str = "Lock Damaged/Seized";
    pub const CODE_11: &'static str = "In Wrong Walk";
    pub const CODE_12: &'static str = "Locked Premises";
    pub const CODE_13: &'static str = "Locked Gate";
    pub const CODE_14: &'static str = "Locked Meter Box";
    pub const CODE_15: &'static str = "Access - Overgrown";
    pub const CODE_16: &'static str = "Noxious Weeds";
    pub const CODE_17: &'static str = "Unsafe Equipment/Location";
    pub const CODE_18: &'static str = "Read Below Previous";
    pub const CODE_19: &'static str = "Consumer Wanted";
    pub const CODE_20: &'static str = "Damaged Equipment/Panel";
    pub const CODE_21: &'static str = "Switched Off";
    pub const CODE_22: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_23: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_24: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_25: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_26: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_27: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_28: &'static str = "Damaged Equipment/Panel";
    pub const CODE_29: &'static str = "Relay Faulty/Damaged";
    pub const CODE_30: &'static str = "Meter Stop Switch On";
    pub const CODE_31: &'static str = "Meter/Equipment Seals Missing";
    pub const CODE_32: &'static str = "Damaged Equipment/Panel";
    pub const CODE_33: &'static str = "Relay Faulty/Damaged";
    pub const CODE_34: &'static str = "Meter Not In Handheld";
    pub const CODE_35: &'static str = "Timeswitch Faulty/Reset Required";
    pub const CODE_36: &'static str = "Meter High/Ladder Required";
    pub const CODE_37: &'static str = "Meter High/Ladder Required";
    pub const CODE_38: &'static str = "Unsafe Equipment/Location";
    pub const CODE_39: &'static str = "Reverse Energy Observed";
    pub const CODE_40: &'static str = "Timeswitch Faulty/Reset Required";
    pub const CODE_41: &'static str = "Faulty Equipment Display/Dials";
    pub const CODE_42: &'static str = "Faulty Equipment Display/Dials";
    pub const CODE_43: &'static str = "Power Outage";
    pub const CODE_44: &'static str = "Unsafe Equipment/Location";
    pub const CODE_45: &'static str = "Readings Failed To Validate";
    pub const CODE_46: &'static str = "Extreme Weather/Hot";
    pub const CODE_47: &'static str = "Refused Access";
    pub const CODE_48: &'static str = "Timeswitch Faulty/Reset Required";
    pub const CODE_49: &'static str = "Wet Paint";
    pub const CODE_50: &'static str = "Wrong Tariff";
    pub const CODE_51: &'static str = "Installation Demolished";
    pub const CODE_52: &'static str = "Access - Blocked";
    pub const CODE_53: &'static str = "Bees/Wasp In Meter Box";
    pub const CODE_54: &'static str = "Meter Box Damaged/Faulty";
    pub const CODE_55: &'static str = "Faulty Equipment Display/Dials";
    pub const CODE_56: &'static str = "Meter Box Damaged/Faulty";
    pub const CODE_57: &'static str = "Timeswitch Faulty/Reset Required";
    pub const CODE_58: &'static str = "Meter Ok - Supply Failure";
    pub const CODE_59: &'static str = "Faulty Equipment Display/Dials";
    pub const CODE_60: &'static str = "Illegal Connection/Equipment Tampered";
    pub const CODE_61: &'static str = "Meter Box Damaged/Faulty";
    pub const CODE_62: &'static str = "Damaged Equipment/Panel";
    pub const CODE_63: &'static str = "Illegal Connection/Equipment Tampered";
    pub const CODE_64: &'static str = "Key Required";
    pub const CODE_65: &'static str = "Wrong Key Provided";
    pub const CODE_66: &'static str = "Lock Damaged/Seized";
    pub const CODE_67: &'static str = "Extreme Weather/Wet";
    pub const CODE_68: &'static str = "Zero Consumption";
    pub const CODE_69: &'static str = "Reading Exceeds Estimate";
    pub const CODE_70: &'static str = "Probe Reports Tampering";
    pub const CODE_71: &'static str = "Probe Read Error";
    pub const CODE_72: &'static str = "Meter/Equipment Changed";
    pub const CODE_73: &'static str = "Low Consumption";
    pub const CODE_74: &'static str = "High Consumption";
    pub const CODE_75: &'static str = "Customer Read";
    pub const CODE_76: &'static str = "Communications Fault";
    pub const CODE_77: &'static str = "Estimation Forecast";
    pub const CODE_78: &'static str = "Null Data";
    pub const CODE_79: &'static str = "Power Outage Alarm";
    pub const CODE_80: &'static str = "Short Interval Alarm";
    pub const CODE_81: &'static str = "Long Interval Alarm";
    pub const CODE_82: &'static str = "CRC Error";
    pub const CODE_83: &'static str = "RAM Checksum Error";
    pub const CODE_84: &'static str = "ROM Checksum Error";
    pub const CODE_85: &'static str = "Data Missing Alarm";
    pub const CODE_86: &'static str = "Clock Error Alarm";
    pub const CODE_87: &'static str = "Reset Occurred";
    pub const CODE_88: &'static str = "Watchdog Timeout Alarm";
    pub const CODE_89: &'static str = "Time Reset Occurred";
    pub const CODE_90: &'static str = "Test pub mode";
    pub const CODE_91: &'static str = "Load Control";
    pub const CODE_92: &'static str = "Added Interval (Data Correction)";
    pub const CODE_93: &'static str = "Replaced Interval (Data Correction)";
    pub const CODE_94: &'static str = "Estimated Interval (Data Correction)";
    pub const CODE_95: &'static str = "Pulse Overflow Alarm";
    pub const CODE_96: &'static str = "Data Out Of Limits";
    pub const CODE_97: &'static str = "Excluded Data";
    pub const CODE_98: &'static str = "Parity Error";
    pub const CODE_99: &'static str = "Energy Type (Register Changed)";
}   

pub mod DATA_STREAM_SUFFIX {
    // Averaged Data Streams
    pub const A: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Average", description: "Import", units: "kWh" };
    pub const D: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Average", description: "Export", units: "kWh" };
    pub const J: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Average", description: "Import", units: "kVArh" };
    pub const P: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Average", description: "Export", units: "kVArh" };
    pub const S: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Average", description: "",       units: "kVAh" };
    // Master Data Streams
    pub const B: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Import", units: "kWh" };
    pub const E: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Export", units: "kWh" };
    pub const K: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Import", units: "kVArh" };
    pub const Q: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Export", units: "kVArh" };
    pub const T: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "",       units: "kVAh" };
    pub const G: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Power Factor", units: "PF" };
    pub const H: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Q Metering", units: "Qh" };
    pub const M: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Par Metering", units: "parh" };
    pub const V: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Master",  description: "Volts or V2h or Amps or A2h", units: "" };
    // Check Meter Streams
    pub const C: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Import", units: "kWh" };
    pub const F: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Export", units: "kWh" };
    pub const L: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Import", units: "kVArh" };
    pub const R: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Export", units: "kVArh" };
    pub const U: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "",       units: "kVAh" };
    pub const Y: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Q Metering",         units: "Qh" };
    pub const W: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Par Metering Path",  units: "" };
    pub const Z: &'static DataStreamSuffix = &DataStreamSuffix { stream: "Check",  description: "Volts or V2h or Amps or A2h",  units: "" };
    // Net Meter Streams
    // AEMO: NOTE THAT D AND J ARE PREVIOUSLY DEFINED
    // "D" = { stream: "Net",    description: "Net", units: "kWh" },
    // "J" = { stream: "Net",    description: "Net", units: "kVArh" }

    pub struct DataStreamSuffix<'a> {
        stream: &'a str,
        description: &'a str,
        units: &'a str
    }
}