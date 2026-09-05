use crate::config::DescriptorType;
use crate::db::UtxoRecord;
use anyhow::{Result, bail};

pub const DUST_THRESHOLD_SATS: u64 = 546;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinSelectionStrategy {
    LargestFirst,
    SmallestFirst,
    ExactMatch,
}

impl std::str::FromStr for CoinSelectionStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "largest-first" | "largestfirst" | "largest" => Ok(CoinSelectionStrategy::LargestFirst),
            "smallest-first" | "smallestfirst" | "smallest" => {
                Ok(CoinSelectionStrategy::SmallestFirst)
            }
            "exact-match" | "exactmatch" | "exact" => Ok(CoinSelectionStrategy::ExactMatch),
            other => bail!(
                "Unknown coin selection strategy: {}. Use 'largest-first', 'smallest-first', or 'exact-match'",
                other
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoinSelectionResult {
    pub selected_utxos: Vec<UtxoRecord>,
    pub total_input_sats: u64,
    pub target_amount_sats: u64,
    pub fee_sats: u64,
    pub change_sats: u64,
    pub create_change_output: bool,
}

/// Estimates transaction virtual size in vbytes based on input/output counts and descriptor type
pub fn estimate_vsize(
    num_inputs: usize,
    num_outputs: usize,
    descriptor_type: DescriptorType,
) -> usize {
    let base_overhead = 11; // version (4), locktime (4), segwit marker & flag (0.5), in/out counts (2)

    let input_vsize = match descriptor_type {
        DescriptorType::Wpkh => 68, // outpoint(36) + scriptSig len(1) + sequence(4) + witness(~107/4 = 27)
        DescriptorType::Tr => 58, // outpoint(36) + scriptSig len(1) + sequence(4) + witness(~65/4 = 17)
    };

    let output_vsize = match descriptor_type {
        DescriptorType::Wpkh => 31, // value(8) + script len(1) + script(22)
        DescriptorType::Tr => 43,   // value(8) + script len(1) + script(34)
    };

    base_overhead + (num_inputs * input_vsize) + (num_outputs * output_vsize)
}

/// Calculate fee given vsize and fee_rate in sat/vB (minimum 1 sat)
pub fn calculate_fee(vsize: usize, fee_rate_sat_per_vb: u64) -> u64 {
    let fee = (vsize as u64) * fee_rate_sat_per_vb;
    if fee == 0 { 1 } else { fee }
}

pub fn select_coins(
    available_utxos: &[UtxoRecord],
    target_amount_sats: u64,
    fee_rate_sat_per_vb: u64,
    descriptor_type: DescriptorType,
    strategy: CoinSelectionStrategy,
) -> Result<CoinSelectionResult> {
    if available_utxos.is_empty() {
        bail!("No UTXOs available for coin selection");
    }

    if target_amount_sats == 0 {
        bail!("Target amount must be greater than 0");
    }

    let mut utxos = available_utxos.to_vec();

    match strategy {
        CoinSelectionStrategy::LargestFirst => {
            utxos.sort_by_key(|b| std::cmp::Reverse(b.amount_sats));
        }
        CoinSelectionStrategy::SmallestFirst => {
            utxos.sort_by_key(|a| a.amount_sats);
        }
        CoinSelectionStrategy::ExactMatch => {
            // First look for an exact single UTXO match (target + fee with 1 output)
            let fee_1in_1out =
                calculate_fee(estimate_vsize(1, 1, descriptor_type), fee_rate_sat_per_vb);
            let target_with_fee = target_amount_sats + fee_1in_1out;

            if let Some(pos) = utxos.iter().position(|u| u.amount_sats == target_with_fee) {
                let selected = vec![utxos[pos].clone()];
                return Ok(CoinSelectionResult {
                    selected_utxos: selected,
                    total_input_sats: target_with_fee,
                    target_amount_sats,
                    fee_sats: fee_1in_1out,
                    change_sats: 0,
                    create_change_output: false,
                });
            }

            // Otherwise, sort by smallest difference >= target_with_fee
            utxos.sort_by(|a, b| {
                let diff_a = if a.amount_sats >= target_with_fee {
                    a.amount_sats - target_with_fee
                } else {
                    u64::MAX - a.amount_sats
                };
                let diff_b = if b.amount_sats >= target_with_fee {
                    b.amount_sats - target_with_fee
                } else {
                    u64::MAX - b.amount_sats
                };
                diff_a.cmp(&diff_b)
            });
        }
    }

    let mut selected = Vec::new();
    let mut total_in = 0u64;

    for utxo in utxos {
        selected.push(utxo.clone());
        total_in += utxo.amount_sats;

        // Try assuming a change output exists (2 outputs: recipient + change)
        let vsize_with_change = estimate_vsize(selected.len(), 2, descriptor_type);
        let fee_with_change = calculate_fee(vsize_with_change, fee_rate_sat_per_vb);

        if total_in >= target_amount_sats + fee_with_change {
            let change = total_in - target_amount_sats - fee_with_change;

            if change >= DUST_THRESHOLD_SATS {
                return Ok(CoinSelectionResult {
                    selected_utxos: selected,
                    total_input_sats: total_in,
                    target_amount_sats,
                    fee_sats: fee_with_change,
                    change_sats: change,
                    create_change_output: true,
                });
            } else {
                // Change is below dust threshold: omit change output and donate to miner fee
                let vsize_no_change = estimate_vsize(selected.len(), 1, descriptor_type);
                let min_fee_no_change = calculate_fee(vsize_no_change, fee_rate_sat_per_vb);
                let actual_fee = total_in - target_amount_sats;

                if actual_fee >= min_fee_no_change {
                    return Ok(CoinSelectionResult {
                        selected_utxos: selected,
                        total_input_sats: total_in,
                        target_amount_sats,
                        fee_sats: actual_fee,
                        change_sats: 0,
                        create_change_output: false,
                    });
                }
            }
        }
    }

    // Check if total without change covers recipient + 1 output fee
    let vsize_1out = estimate_vsize(selected.len(), 1, descriptor_type);
    let fee_1out = calculate_fee(vsize_1out, fee_rate_sat_per_vb);

    if total_in >= target_amount_sats + fee_1out {
        let fee = total_in - target_amount_sats;
        return Ok(CoinSelectionResult {
            selected_utxos: selected,
            total_input_sats: total_in,
            target_amount_sats,
            fee_sats: fee,
            change_sats: 0,
            create_change_output: false,
        });
    }

    bail!(
        "Insufficient funds: have {} sats, need at least {} sats (target {} + estimated fee {})",
        total_in,
        target_amount_sats + fee_1out,
        target_amount_sats,
        fee_1out
    );
}
