use anyhow::{Context, Result};

use pallas::{
    codec::minicbor::encode,
    ledger::{
        addresses::PaymentKeyHash,
        primitives::conway::{Constr, PlutusData},
    },
    txbuilder::{Output, StagingTransaction},
};

use crate::model::hydra::contract::hydra_validator::HydraValidator;

use super::{input::InputWrapper, output::OutputWrapper, script_registry::ScriptRegistry};

pub struct CommitTx {
    network_id: u8,
    script_registry: ScriptRegistry,
    head_id: Vec<u8>,
    party: Vec<u8>,
    initial_input: (InputWrapper, Output, PaymentKeyHash),
    commit_inputs: Vec<(InputWrapper, OutputWrapper)>,
}

impl CommitTx {
    pub fn build_tx(&self) -> Result<StagingTransaction> {
        let commit_output = build_base_commit_output(
            self.commit_inputs
                .iter()
                .map(|(_, o)| o.inner.clone())
                .collect(),
            self.network_id,
        )
        .context("Failed to construct base commit output")?
        .set_inline_datum(self.build_commit_datum()?);

        let tx_builder = StagingTransaction::new().output(commit_output);

        Ok(tx_builder)
    }

    fn build_commit_datum(&self) -> Result<Vec<u8>> {
        let fields = vec![
            PlutusData::BoundedBytes(self.party.clone().into()),
            PlutusData::Array(
                self.commit_inputs
                    .clone()
                    .into_iter()
                    .map(|(commit_input, commit_input_output)| {
                        let output_data: PlutusData = commit_input_output.into();
                        let mut output_bytes = Vec::new();
                        encode(&output_data, &mut output_bytes)?;
                        Ok(PlutusData::Constr(Constr {
                            tag: 121,
                            any_constructor: None,
                            fields: vec![
                                commit_input.into(),
                                PlutusData::BoundedBytes(output_bytes.into()),
                            ],
                        }))
                    })
                    .collect::<Result<Vec<PlutusData>, anyhow::Error>>()?,
            ),
            PlutusData::BoundedBytes(self.head_id.clone().into()),
        ];
        let data = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields,
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&data, &mut bytes).context("Failed to encode plutus data in CBOR")?;

        Ok(bytes)
    }

    fn build_redeemer(&self) -> Result<Vec<u8>> {
        let redeemer_data = PlutusData::Constr(Constr {
            tag: 122,
            any_constructor: None,
            fields: vec![PlutusData::Array(
                self.commit_inputs
                    .iter()
                    .map(|(input, _)| input.into())
                    .collect::<Vec<_>>(),
            )],
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&redeemer_data, &mut bytes).context("Failed to encode plutus data in CBOR")?;
        Ok(bytes)
    }
}

fn build_base_commit_output(outputs: Vec<Output>, network_id: u8) -> Result<Output> {
    let address = HydraValidator::VDeposit.to_address(network_id);
    let lovelace = outputs.iter().fold(0, |acc, o| acc + o.lovelace);
    let mut commit_output = Output::new(address, lovelace);
    for output in outputs {
        if let Some(output_assets) = output.assets {
            for (policy, assets) in output_assets.iter() {
                for (name, amount) in assets {
                    commit_output = commit_output
                        .add_asset(policy.0.into(), name.0.clone(), amount.clone())
                        .context("Failed to add asset to commit output")?;
                }
            }
        }
    }

    Ok(commit_output)
}

mod tests {
    use pallas::{crypto::hash::Hash, ledger::addresses::Address, txbuilder::Input};

    use super::*;

    #[test]
    fn test_build_commit_datum() {
        let datum = get_commit()
            .build_commit_datum()
            .expect("Failed to build commit datum");

        assert_eq!(hex::encode(datum), "d8799f58203302e982ae2514964bcd2b2d7187277a2424e44b553efafaf786677ff5db9a5e9fd8799fd8799fd8799f582008e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948ff182eff5f5840d8799fd8799fd8799f581c299650d431de775c65eed15c122aa975237e5b4a235a596c0b5edcf3ffd8799fd8799fd8799f581c496ab2039877b6386666a3d6515823e38eaf04c7c0a46c09f7f939ebfd6effffffffa140a1401a00989680d87980d87a80ffffffd8799fd8799fd8799f58205a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8ff03ff5f5840d8799fd8799fd8799f581cb9343d7c6de66302960110db759633e7bc4ce1ef8d3faa2386938dedffd8799fd8799fd8799f581c8202e1e5de55b5025e8a4afe4558239cea10057e97825c3d34623ad28d1fffffffffa140a1401a05c81a40d87980d87a80ffffffd8799fd8799fd8799f58207663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0ff07ff5f5840d8799fd8799fd8799f581cee1a6e03cbd9ede8d40ae6fdb6ab51a9de7b603af218730fe5e56d35ffd8799fd8799fd8799f581c504be8610c412878fb01c9ef7758239f1e6bbd1c1aeafe3c6c6c8f50d142ffffffffa140a1401a030a32c0d87980d87a80ffffffff581c2505642019121d9b2d92437d8b8ea493bacfcb4fb535013b70e7f528ff");
    }

    #[test]
    fn test_build_redeemer() {
        let redeemer = get_commit()
            .build_redeemer()
            .expect("Failed to build redeemer");

        assert_eq!(hex::encode(redeemer), "d87a9f9fd8799fd8799f582008e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948ff182effd8799fd8799f58205a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8ff03ffd8799fd8799f58207663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0ff07ffffff");
    }

    // This CommitTx uses the following preview transaction: d00b6b2c3920c8836ca0bce2fe4f662bd68c3d49dca743831fd9328b44260908
    fn get_commit() -> CommitTx {
        let head_id = hex::decode("2505642019121D9B2D92437D8B8EA493BACFCB4FB535013B70E7F528")
            .expect("Failed to decode head_id");
        let party = hex::decode("3302e982ae2514964bcd2b2d7187277a2424e44b553efafaf786677ff5db9a5e")
            .expect("Failed to decode party");
        let initial_input: (InputWrapper, Output, PaymentKeyHash) = (
            Input::new(
                Hash::from(
                    hex::decode("ef61c1686e77e6004f7e9913d20d0598e8cc5e661a559086a84dfafaafdc7818")
                        .expect("Failed to decode txid")
                        .as_slice(),
                ),
                1,
            )
            .into(),
            Output::new(
                Address::from_bech32(
                    "addr_test1wqh6eqv6ra83fc5k88g5zs3q62sck64adw8ygnvg6rw63lc70pepc",
                )
                .expect("failed to decode bech32"),
                1290000,
            ),
            Hash::from(
                hex::decode("2fac819a1f4f14e29639d1414220d2a18b6abd6b8e444d88d0dda8ff")
                    .expect("failed to decode key hash")
                    .as_slice(),
            ),
        );

        CommitTx {
                network_id: 0,
                script_registry: ScriptRegistry {
                    initial_reference: initial_input.0.clone().into(),
                    commit_reference: initial_input.0.clone().into(),
                    head_reference: initial_input.0.clone().into(),
                },
                head_id,
                party,
                initial_input,
                commit_inputs: vec![(
                    Input::new(
                        Hash::from(
                            hex::decode(
                                "08e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948",
                            )
                            .expect("Failed to decode txid")
                            .as_slice(),
                        ),
                        46,
                    )
                    .into(),
                    Output::new(
                        Address::from_bech32(
                            "addr_test1qq5ev5x5x808whr9amg4cy32496jxljmfg345ktvpd0deu6fd2eq8xrhkcuxve4r6eg78r40qnrupfrvp8mljw0tl4hqe383dk"
                        )
                        .expect("failed to decode bech32"),
                        10000000
                    ).into(),
                ),
                (
                    Input::new(
                        Hash::from(
                            hex::decode(
                                "5a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8",
                            )
                            .expect("Failed to decode txid")
                            .as_slice(),
                        ),
                        3,
                    )
                    .into(),
                    Output::new(
                        Address::from_bech32(
                            "addr_test1qzung0tudhnxxq5kqygdkavkx0nmcn8pa7xnl23rs6fcmmvzqts7thj4k5p9azj2lezee6ssq4lf0qju856xywkj350sew4adl"
                        )
                        .expect("failed to decode bech32"),
                        97000000
                    ).into(),
                ),
                (
                    Input::new(
                        Hash::from(
                            hex::decode(
                                "7663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0"
                            )
                            .expect("failed to decode tx_id")
                            .as_slice()
                        ),
                        7
                    )
                    .into(),
                    Output::new(
                        Address::from_bech32(
                            "addr_test1qrhp5msre0v7m6x5ptn0md4t2x5au7mq8tepsuc0uhjk6d2sf05xzrzp9pu0kqwfaame78nth5wp46h783kxer6s69pq74eyeg"
                        )
                        .expect("failed to decode bech32"),
                        51000000
                    ).into()
                )
                ],
            }
    }
}