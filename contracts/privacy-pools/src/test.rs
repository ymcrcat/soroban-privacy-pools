#![cfg(test)]
use super::*;
use ark_bls12_381::{Fq, Fq2};
use ark_serialize::CanonicalSerialize;
use core::str::FromStr;
use soroban_sdk::{
    vec, Address, Bytes, BytesN, Env, String,
    crypto::bls12_381::{G1Affine, G2Affine, G1_SERIALIZED_SIZE, G2_SERIALIZED_SIZE, Fr},
    U256
};
use soroban_sdk::testutils::Address as TestAddress;

fn g1_from_coords(env: &Env, x: &str, y: &str) -> G1Affine {
    let ark_g1 = ark_bls12_381::G1Affine::new(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
    let mut buf = [0u8; G1_SERIALIZED_SIZE];
    ark_g1.serialize_uncompressed(&mut buf[..]).unwrap();
    G1Affine::from_array(env, &buf)
}

fn g2_from_coords(env: &Env, x1: &str, x2: &str, y1: &str, y2: &str) -> G2Affine {
    let x = Fq2::new(Fq::from_str(x1).unwrap(), Fq::from_str(x2).unwrap());
    let y = Fq2::new(Fq::from_str(y1).unwrap(), Fq::from_str(y2).unwrap());
    let ark_g2 = ark_bls12_381::G2Affine::new(x, y);
    let mut buf = [0u8; G2_SERIALIZED_SIZE];
    ark_g2.serialize_uncompressed(&mut buf[..]).unwrap();
    G2Affine::from_array(env, &buf)
}

fn init_vk(env: &Env) -> Bytes {
    let alphax = "2625583050305146829700663917277485398332586266229739236073977691599912239208704058548731458555934906273399977862822";
    let alphay = "1155364156944807367912876641032696519500054551629402873339575774959620483194368919563799050765095981406853619398751";
    
    
    let betax1 = "1659696755509039809248937927616726274238080235224171061036366585278216098417245587200210264410333778948851576160490";
    let betax2 = "1338363397031837211155983756179787835339490797745307535810204658838394402900152502268197396587061400659003281046656";
    let betay1 = "1974652615426136516341494326987376616840373177388374023461177997087381634383568759591087499459321812809521924259354";
    let betay2 = "3301884318087924474550898163462840036865878131635519297186391370517333773367262804074867347346141727012544462046142";
    
    
    let gammax1 = "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160";
    let gammax2 = "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758";
    let gammay1 = "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905";
    let gammay2 = "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582";
    
    
    let deltax1 = "3382327709967179305449866439930078454748364767249082720038545797989620288380206486763165075451797528765360519177593";
    let deltax2 = "3773195663498217966966081584690172808393354403144769091023164955810583345086500360825403939295222519084338722582578";
    let deltay1 = "1052818418041168469755198458393791449667480584316278322917379997019363027521182658723526172335998912673597768646340";
    let deltay2 = "3439746335387638714469116231154701265034453108578594311881882423838496374772966663248811073113423658545866927011324";
    
    
    let ic0x = "1429614514405605701316821410060713976491573356547706820430125512182443913956177831440946230912781831391842190950269";
    let ic0y = "2595245330717167843240408561437713436226305887613714615478236538597928352802205479515983503568437719268312968597187";
    
    
    let ic1x = "699922020262726318547273768369196747750943039862935657319907208596421370804231621163221655994936087961528530443398";
    let ic1y = "1183789540308001547069705756290550538783265182880422084641597548728082450785620403432008991010051245851892233685780";

    let vk = VerificationKey {
        alpha: g1_from_coords(env, alphax, alphay),
        beta: g2_from_coords(env, betax1, betax2, betay1, betay2),
        gamma: g2_from_coords(env, gammax1, gammax2, gammay1, gammay2),
        delta: g2_from_coords(env, deltax1, deltax2, deltay1, deltay2),
        ic: Vec::from_array(
            &env,
            [
                g1_from_coords(env, ic0x, ic0y),
                g1_from_coords(env, ic1x, ic1y),
                // g1_from_coords(env, ic2x, ic2y),
            ],
        ),
    };
    
    return vk.to_bytes(env);
}

fn init_proof(env: &Env) -> Bytes {
    let pi_ax = "1744337292642413355698725004362228823547090398268290807453528244238473222941708245032975078295502937790537712565579";
    let pi_ay = "3852420325385363031060315407402029196046197611162175489464327159954937083956548374239448059077789663842203871378909";
    
    
    let pi_bx1 = "362267734205382607163250267783985556247339978866647687105514093472298662993458212405689773173713679261452890116018";
    let pi_bx2 = "1175735518810204610514848218272344512665804488575924693066792946062049951910578364370169662085109290080386389864363";
    let pi_by1 = "555616434041364540333440447141742242910319129596048406940062110953555284556157902169263449404565191560956305334576";
    let pi_by2 = "3508139269038498641651777110964502483179638602625820565589333717612766252841198722701472004774487537992222477323964";
    
    
    let pi_cx = "3040383222314091205689398841737962739924011073008763299216475806132646605421491413117818537231855049073347281265798";
    let pi_cy = "1426389548091897369807778370018184082083115905257261249836859031492729345032943085696956269824869614362635166163200";

    // Construct the proof from the pre-computed components
    let proof = Proof {
        a: g1_from_coords(env, &pi_ax, &pi_ay),
        b: g2_from_coords(env, &pi_bx1, &pi_bx2, &pi_by1, &pi_by2),
        c: g1_from_coords(env, &pi_cx, &pi_cy),
    };

    return proof.to_bytes(env);
}

fn init_pub_signals(env: &Env) -> Bytes {
    let public_0 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x25, 0xab, 0x19, 0xc8, 0xe0, 0xfd, 0x16, 0x5d, 0x61, 0x6e, 0xf8, 0xdb, 0xb7, 0xfa, 0x8e, 0x77, 0xd4, 0xb0, 0x42, 0xac, 0x77, 0x24, 0xc2, 0x1e, 0x12, 0x3f, 0x6d, 0x1f, 0xc7, 0x9b, 0xc1, 0xab]));
    // Create output vector for verification:
    let output = Vec::from_array(&env, [Fr::from_u256(public_0)]);
    
    let pub_signals = PublicSignals {
        pub_signals: output
    };

    return pub_signals.to_bytes(env);
}

fn init_erronous_pub_signals(env: &Env) -> Bytes {
    let pub_signals = PublicSignals {
        pub_signals: Vec::from_array(env, [Fr::from_u256(U256::from_u32(env, 34))])
    };

    return pub_signals.to_bytes(env);
}

#[test]
fn test_deposit_and_withdraw() {
    let env = Env::default();
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env),));
    
    // Create test addresses
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);

    // Test initial balance
    assert_eq!(client.get_balance(), 0);

    // Test deposit
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    
    // Mock authentication for alice
    env.mock_all_auths();
    client.deposit(&alice, &commitment);
    
    // Check balance after deposit
    assert_eq!(client.get_balance(), FIXED_AMOUNT);
    // Check commitments
    let commitments = client.get_commitments();
    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments.get(0).unwrap(), commitment);

    // Test withdraw
    let proof = init_proof(&env);
    let pub_signals = init_pub_signals(&env);
    let pub_signals_struct = PublicSignals::from_bytes(&env, &pub_signals);
    let nullifier = pub_signals_struct.pub_signals.get(0).unwrap().to_bytes();

    let result = client.withdraw(&bob, &proof, &pub_signals);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, ERROR_WITHDRAW_SUCCESS)
        ]
    );

    env.cost_estimate().budget().print();

    // Check balance after withdrawal
    assert_eq!(client.get_balance(), 0);

    // Check nullifiers
    let nullifiers = client.get_nullifiers();
    assert_eq!(nullifiers.len(), 1);
    assert_eq!(nullifiers.get(0).unwrap(), nullifier);

    
}

#[test]
fn test_deposit_and_withdraw_wrong_proof() {
    let env = Env::default();
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env),));
    
    // Create test addresses
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);

    // Test initial balance
    assert_eq!(client.get_balance(), 0);

    // Test deposit
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    
    // Mock authentication for alice
    env.mock_all_auths();
    client.deposit(&alice, &commitment);
    
    // Check balance after deposit
    assert_eq!(client.get_balance(), FIXED_AMOUNT);
    // Check commitments
    let commitments = client.get_commitments();
    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments.get(0).unwrap(), commitment);

    // Test withdraw
    let proof = init_proof(&env);
    let pub_signals = init_erronous_pub_signals(&env);
    
    let result = client.withdraw(&bob, &proof, &pub_signals);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, ERROR_COIN_OWNERSHIP_PROOF)
        ]
    );
    assert_eq!(client.get_balance(), FIXED_AMOUNT);
    let nullifiers = client.get_nullifiers();
    assert_eq!(nullifiers.len(), 0);


    // env.cost_estimate().budget().print();
}

#[test]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env),));
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);

    let bob = Address::generate(&env);
    let proof = init_proof(&env);
    let pub_signals = init_pub_signals(&env);
    // Attempt to withdraw with zero balance
    env.mock_all_auths();
    let result = client.withdraw(&bob, &proof, &pub_signals);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, ERROR_INSUFFICIENT_BALANCE)
        ]
    );

    // Ensure nullifier was not stored when withdrawal failed
    assert_eq!(client.get_nullifiers().len(), 0);
}

#[test]
fn test_reuse_nullifier() {
    let env = Env::default();
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env),));
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // First deposit
    let commitment = BytesN::from_array(&env, &[4u8; 32]);
    env.mock_all_auths();
    client.deposit(&alice, &commitment);

    // First withdraw
    let proof = init_proof(&env);
    let pub_signals = init_pub_signals(&env);
    client.withdraw(&bob, &proof, &pub_signals);

    // Second deposit
    let commitment2 = BytesN::from_array(&env, &[6u8; 32]);
    client.deposit(&alice, &commitment2);
    // Attempt to reuse nullifier
    let result = client.withdraw(&bob, &proof, &pub_signals);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, ERROR_NULLIFIER_USED)
        ]
    );
}

#[test]
fn test_keccak256_compatibility() {
    let env = Env::default();
    
    // Test vectors matching the JavaScript reference values
    // These should match the outputs from generate_keccak_reference.js
    
    // Test 1: Single byte 0x00
    let input1 = Bytes::from_array(&env, &[0x00]);
    let hash1 = env.crypto().keccak256(&input1);
    let expected1 = "bc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a";
    assert_eq!(hex::encode(hash1.to_array()), expected1);
    
    // Test 2: Single byte 0xFF
    let input2 = Bytes::from_array(&env, &[0xFF]);
    let hash2 = env.crypto().keccak256(&input2);
    let expected2 = "8b1a944cf13a9a1c08facb2c9e98623ef3254d2ddb48113885c3e8e97fec8db9";
    assert_eq!(hex::encode(hash2.to_array()), expected2);
    
    // Test 3: Two bytes 0x0102
    let input3 = Bytes::from_array(&env, &[0x01, 0x02]);
    let hash3 = env.crypto().keccak256(&input3);
    let expected3 = "22ae6da6b482f9b1b19b0b897c3fd43884180a1c5ee361e1107a1bc635649dda";
    assert_eq!(hex::encode(hash3.to_array()), expected3);
    
    // Test 4: 32 bytes all zeros
    let input4 = Bytes::from_array(&env, &[0u8; 32]);
    let hash4 = env.crypto().keccak256(&input4);
    let expected4 = "290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563";
    assert_eq!(hex::encode(hash4.to_array()), expected4);
    
    // Test 5: Merkle tree style (64 bytes: left=1, right=2) - MOST IMPORTANT TEST
    // This matches exactly what the Soroban LeanIMT hash_pair function does
    let mut combined = [0u8; 64];
    combined[31] = 0x01; // Left value = 1 (32 bytes)
    combined[63] = 0x02; // Right value = 2 (32 bytes)
    let input5 = Bytes::from_slice(&env, &combined);
    let hash5 = env.crypto().keccak256(&input5);
    let expected5 = "e90b7bceb6e7df5418fb78d8ee546e97c83a08bbccc01a0644d599ccd2a7c2e0";
    assert_eq!(hex::encode(hash5.to_array()), expected5);
    
    // Test 6: Empty input
    let input6 = Bytes::from_array(&env, &[]);
    let hash6 = env.crypto().keccak256(&input6);
    let expected6 = "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
    assert_eq!(hex::encode(hash6.to_array()), expected6);
    
    // All tests passed - Soroban's keccak256 matches the js-sha3 reference implementation
}
