// Storing pre-determined motifs as consts for ease of use
pub const K90D: &'static str = "AGCUUACUG";
pub const O3WJ: [&'static str; 3] = ["UACUAA", "UUGUUUC", "GUGUA"];
pub const O4WJ: [&'static str; 4] = [
    "AGGGUUAGCC", 
    "CAUACCGCAA", 
    "AGUGAAAGUU", 
    "GGUCGAUCAC"
];
pub const KL_HAIRPINS: [[&'static str; 2]; 4] = [
    ["GGUCCUAAGU", "CCAGGAUUGA"],
    ["CUACCCUAGG", "GAUGGGAUCC"],
    ["ACCUCGUACA", "UGGAGCAUGU"],
    ["UGGUAAUCGA", "ACCAUUAGCU"],
];

