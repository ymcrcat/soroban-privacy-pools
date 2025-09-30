#![cfg(test)]
use super::*;
use ark_bls12_381::{Fq, Fq2};
use ark_serialize::CanonicalSerialize;
use core::str::FromStr;
use soroban_sdk::{
    vec, Address, Bytes, BytesN, Env, String,
    crypto::bls12_381::{G1Affine, G2Affine, G1_SERIALIZED_SIZE, G2_SERIALIZED_SIZE, Fr},
    U256, symbol_short
};
use soroban_sdk::testutils::Address as TestAddress;

// Mock token contract for testing
#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn initialize(env: &Env, admin: Address, decimal: u32, name: String, symbol: String) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("decimal"), &decimal);
        env.storage().instance().set(&symbol_short!("name"), &name);
        env.storage().instance().set(&symbol_short!("symbol"), &symbol);
    }

    pub fn mint(env: &Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        
        let current_balance = env.storage().instance().get(&to).unwrap_or(0);
        env.storage().instance().set(&to, &(current_balance + amount));
    }

    pub fn balance(env: &Env, id: Address) -> i128 {
        env.storage().instance().get(&id).unwrap_or(0)
    }

    pub fn transfer(env: &Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        
        let from_balance = env.storage().instance().get(&from).unwrap_or(0);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        
        let to_balance = env.storage().instance().get(&to).unwrap_or(0);
        env.storage().instance().set(&from, &(from_balance - amount));
        env.storage().instance().set(&to, &(to_balance + amount));
    }
}

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
    
    
    let deltax1 = "3286112056055901745783763610785013808339608449720480145481113394642110789742171495819638103767342488641015809697442";
    let deltax2 = "3659614273286313992469938254186369271776064865510287043938386272195007987104762581831348996867215652241357736915416";
    let deltay1 = "296798597200968764039855317532338800887547655704345479531957042324576659048978349874698362495851637268098914634981";
    let deltay2 = "3145194167570807349073454036462828095420982769946665745057368714319806878087767482119915857283479776589844129080169";
    
    
    let ic0x = "3365201128768285122593922246439163400077160005579927669663554216358372814666157405994502762300268897534547476884887";
    let ic0y = "2571857116928012707805771465903505670387731108688654111474313656178409546287116367452470976588658128759240888143619";
    
    
    let ic1x = "3241560128078625228977339242839078845935873577479477291727559158135109287056933398913964365068298746466590053504042";
    let ic1y = "3206167993742736528242215230426200451228323597556130750039823493352510829244159750044330655198644622969589089519819";
    
    
    let ic2x = "1011805795846263147216753685893296289028620210866908761769380521362573606365468723612940794396332419458961656310360";
    let ic2y = "3405493298307978042121272927400074691171614454260134070645764867627451521102146947690272768329589424083144114238704";
    
    
    let ic3x = "285010110909528813634996373638449888362264283016232687707492488392979573797563466877756491398035174305543101371708";
    let ic3y = "1897004056282288582944120053504075928239225499402919715201928553875065921289606418674027053816330892046179631004263";

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
                g1_from_coords(env, ic2x, ic2y),
                g1_from_coords(env, ic3x, ic3y),
            ],
        ),
    };
    
    return vk.to_bytes(env);
}

fn init_proof(env: &Env) -> Bytes {
    let pi_ax = "623874331280755318675032028406937065977725486252325486511972745459164911332858317134663761205438051903871415266594";
    let pi_ay = "2351505213654826766024240387432194041177606772003164057376315112368186928111937080646159805750758330313591011353323";
    
    
    let pi_bx1 = "3414965805144266469848732520230200070280272613616656676214223685201956233859774915412620537820406336959526884104199";
    let pi_bx2 = "3609731732797990147825878518213556904262746586642259361382733341256064588633959372736831050227039446670621597405445";
    let pi_by1 = "1323221690716023297808669808817894559987846471410382577943086306469821853340399493869813204523404236318081373227344";
    let pi_by2 = "1738568031452103058775637282571705753342283996388095844643100714649217960690663682749313691572484001594046739373231";
    
    
    let pi_cx = "1602028708031377278044991090092661107773050627719437015908830864673473524500342119845170476959740428797897573767945";
    let pi_cy = "152663850280813890470460040537231389750781224281293920573556153853091801399544474324447572068229527992181302454550";

    // Construct the proof from the pre-computed components
    let proof = Proof {
        a: g1_from_coords(env, &pi_ax, &pi_ay),
        b: g2_from_coords(env, &pi_bx1, &pi_bx2, &pi_by1, &pi_by2),
        c: g1_from_coords(env, &pi_cx, &pi_cy),
    };

    return proof.to_bytes(env);
}

fn init_pub_signals(env: &Env) -> Bytes {
    let public_0 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x34, 0x8b, 0x9d, 0x76, 0x54, 0x17, 0xed, 0x40, 0x9f, 0x98, 0xdb, 0x0c, 0x1e, 0x94, 0xa2, 0xba, 0x73, 0x41, 0xc9, 0xd2, 0xb3, 0xf5, 0x53, 0x20, 0xa9, 0xe2, 0x92, 0xd9, 0xab, 0x13, 0xa1, 0x28]));
    let public_1 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0x9a, 0xca, 0x00]));
    let public_2 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x58, 0x4a, 0xc7, 0x9d, 0xad, 0x91, 0x7f, 0xc6, 0x35, 0x38, 0xfc, 0x36, 0x28, 0x9b, 0x8b, 0x11, 0x6e, 0xf9, 0x05, 0x62, 0x81, 0x28, 0xfb, 0x2e, 0xdc, 0x62, 0xb2, 0x75, 0xf7, 0xb3, 0x57, 0x39]));
    
    // Create output vector for verification:
    let output = Vec::from_array(&env, [Fr::from_u256(public_0), Fr::from_u256(public_1), Fr::from_u256(public_2)]);
    
    let pub_signals = PublicSignals {
        pub_signals: output
    };

    return pub_signals.to_bytes(env);
}

fn init_erronous_pub_signals(env: &Env) -> Bytes {
    let public_0 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x65, 0x18, 0x92, 0xef, 0x37, 0x4f, 0x78, 0x93, 0x82, 0x36, 0xd4, 0x83, 0x2b, 0x62, 0xd3, 0x5f, 0xb7, 0x9c, 0x54, 0xf8, 0x72, 0xe3, 0x0f, 0x5a, 0xa9, 0xab, 0xf9, 0xe6, 0xab, 0x15, 0xcb, 0x41]));
    let public_1 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0x9a, 0xca, 0x00]));
    let public_2 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x43, 0xc7, 0x5b, 0x13, 0x4d, 0x38, 0x9a, 0x5f, 0x97, 0x8c, 0xec, 0x2a, 0x75, 0x91, 0x10, 0xe9, 0x9d, 0x1b, 0x9b, 0x7b, 0xe0, 0x34, 0x45, 0xbd, 0xb9, 0x64, 0xd3, 0x43, 0x92, 0xc5, 0x79, 0x63]));
    
    // Create output vector for verification:
    let output = Vec::from_array(&env, [Fr::from_u256(public_0), Fr::from_u256(public_1), Fr::from_u256(public_2)]);
    
    let pub_signals = PublicSignals {
        pub_signals: output
    };

    return pub_signals.to_bytes(env);
}

fn setup_test_environment(env: &Env) -> (Address, Address, Address) {
    // Deploy mock token
    let token_admin = Address::generate(env);
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(env, &token_id);
    
    // Initialize token
    token_client.initialize(
        &token_admin,
        &7u32,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST")
    );
    
    // Deploy privacy pools contract
    let privacy_pools_id = env.register(PrivacyPoolsContract, (init_vk(env), token_id.clone()));
    
    (token_id, privacy_pools_id, token_admin)
}

#[test]
fn test_deposit_and_withdraw_correct_proof() {
    let env = Env::default();
    let (token_id, contract_id, _token_admin) = setup_test_environment(&env);
    env.cost_estimate().budget().print();
    
    // Create test addresses
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);
    let token_client = MockTokenClient::new(&env, &token_id);

    // Mint tokens to alice
    env.mock_all_auths();
    token_client.mint(&alice, &1000000000);

    // Test initial balance
    assert_eq!(client.get_balance(), 0);
    assert_eq!(token_client.balance(&alice), 1000000000);

    // Test deposit
    let commitment = BytesN::from_array(&env, &[
        0x3f, 0xf8, 0x11, 0x53, 0xc6, 0x4a, 0x52, 0x86,
        0xc2, 0x77, 0x42, 0x1e, 0xe1, 0x74, 0xca, 0x86,
        0xfb, 0xd2, 0xa1, 0x80, 0x62, 0x69, 0x31, 0x66,
        0xef, 0xb4, 0x65, 0x0c, 0xe2, 0xf1, 0x72, 0x91
    ]);
    
    // Mock authentication for alice
    env.mock_all_auths();
    client.deposit(&alice, &commitment);
    
    // Check commitments
    let commitments = client.get_commitments();
    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments.get(0).unwrap(), commitment);

    // Check balances after deposit
    assert_eq!(token_client.balance(&alice), 0); // Alice's balance should be 0
    assert_eq!(token_client.balance(&contract_id), 1000000000); // Contract should have the tokens

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

    // Check balances after withdrawal
    assert_eq!(token_client.balance(&bob), 1000000000); // Bob should have the tokens
    assert_eq!(token_client.balance(&contract_id), 0); // Contract should have 0 tokens

    // Check nullifiers
    let nullifiers = client.get_nullifiers();
    assert_eq!(nullifiers.len(), 1);
    assert_eq!(nullifiers.get(0).unwrap(), nullifier);
}

#[test]
fn test_deposit_and_withdraw_wrong_proof() {
    let env = Env::default();
    let (token_id, contract_id, _token_admin) = setup_test_environment(&env);
    
    // Create test addresses
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);
    let token_client = MockTokenClient::new(&env, &token_id);

    // Mint tokens to alice
    env.mock_all_auths();
    token_client.mint(&alice, &1000000000);

    // Test initial balance
    assert_eq!(client.get_balance(), 0);
    assert_eq!(token_client.balance(&alice), 1000000000);

    // Test deposit
    let commitment = BytesN::from_array(&env, &[
        0x3f, 0xf8, 0x11, 0x53, 0xc6, 0x4a, 0x52, 0x86,
        0xc2, 0x77, 0x42, 0x1e, 0xe1, 0x74, 0xca, 0x86,
        0xfb, 0xd2, 0xa1, 0x80, 0x62, 0x69, 0x31, 0x66,
        0xef, 0xb4, 0x65, 0x0c, 0xe2, 0xf1, 0x72, 0x91
    ]);

    // Mock authentication for alice
    env.mock_all_auths();
    client.deposit(&alice, &commitment);
    
    // Check commitments
    let commitments = client.get_commitments();
    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments.get(0).unwrap(), commitment);

    // Test withdraw with wrong proof
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
    
    // Check that balances are unchanged (withdrawal failed)
    assert_eq!(token_client.balance(&bob), 0); // Bob should still have 0
    assert_eq!(token_client.balance(&contract_id), 1000000000); // Contract should still have tokens
    
    let nullifiers = client.get_nullifiers();
    assert_eq!(nullifiers.len(), 0); // No nullifiers should be stored
}

#[test]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    let (_token_id, contract_id, _token_admin) = setup_test_environment(&env);
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
    let (token_id, contract_id, _token_admin) = setup_test_environment(&env);
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);
    let token_client = MockTokenClient::new(&env, &token_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Mint tokens to alice
    env.mock_all_auths();
    token_client.mint(&alice, &2000000000); // Mint enough for two deposits

    // First deposit
    let commitment1 = BytesN::from_array(&env, &[
        0x3f, 0xf8, 0x11, 0x53, 0xc6, 0x4a, 0x52, 0x86,
        0xc2, 0x77, 0x42, 0x1e, 0xe1, 0x74, 0xca, 0x86,
        0xfb, 0xd2, 0xa1, 0x80, 0x62, 0x69, 0x31, 0x66,
        0xef, 0xb4, 0x65, 0x0c, 0xe2, 0xf1, 0x72, 0x91
    ]);
    env.mock_all_auths();
    client.deposit(&alice, &commitment1);

    // First withdraw
    let proof = init_proof(&env);
    let pub_signals = init_pub_signals(&env);
    env.mock_all_auths();
    client.withdraw(&bob, &proof, &pub_signals);

    // Second deposit
    let commitment2 = BytesN::from_array(&env, &[6u8; 32]);
    env.mock_all_auths();
    client.deposit(&alice, &commitment2);
    
    // Attempt to reuse nullifier
    env.mock_all_auths();
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
fn test_contract_initialization() {
    let env = Env::default();
    let token_address = Address::generate(&env);
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env), token_address.clone()));
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);
    
    // Test that contract initializes correctly
    let merkle_root = client.get_merkle_root();
    let merkle_depth = client.get_merkle_depth();
    let commitment_count = client.get_commitment_count();
    let commitments = client.get_commitments();
    let nullifiers = client.get_nullifiers();
    
    // Verify initial state
    assert_eq!(merkle_depth, 2);
    assert_eq!(commitment_count, 0);
    assert_eq!(commitments.len(), 0);
    assert_eq!(nullifiers.len(), 0);
    
    // Merkle root should be initialized (not all zeros)
    assert_ne!(merkle_root, BytesN::from_array(&env, &[0u8; 32]));
}

#[cfg(feature = "test_hash")]
#[test]
fn test_hash_method() {
    let env = Env::default();
    let token_address = Address::generate(&env);
    let contract_id = env.register(PrivacyPoolsContract, (init_vk(&env), token_address));
    let client = PrivacyPoolsContractClient::new(&env, &contract_id);
    
    // Should execute without panicking
    client.test_hash();
}
