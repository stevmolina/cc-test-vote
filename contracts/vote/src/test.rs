#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, VoteClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(Vote, ());
    let client = VoteClient::new(&env, &id);
    (env, client)
}

#[test]
fn votes_are_counted_per_project() {
    let (env, client) = setup();
    let (a, b, c) = (Address::generate(&env), Address::generate(&env), Address::generate(&env));
    let p1 = String::from_str(&env, "Proof of Thanks");
    let p2 = String::from_str(&env, "Tip Jar");

    assert_eq!(client.results().len(), 0);
    assert_eq!(client.vote(&a, &p1), 1);
    assert_eq!(client.vote(&b, &p2), 1);
    assert_eq!(client.vote(&c, &p1), 2);

    let r = client.results();
    assert_eq!(r.len(), 2);
    assert_eq!(r.get(0).unwrap(), Tally { project: p1.clone(), votes: 2 });
    assert_eq!(r.get(1).unwrap(), Tally { project: p2, votes: 1 });
}

#[test]
fn voted_for_remembers_choice() {
    let (env, client) = setup();
    let a = Address::generate(&env);
    let p = String::from_str(&env, "Tip Jar");
    assert_eq!(client.voted_for(&a), None);
    client.vote(&a, &p);
    assert_eq!(client.voted_for(&a), Some(p));
}

#[test]
fn vote_requires_voter_auth() {
    let (env, client) = setup();
    let a = Address::generate(&env);
    client.vote(&a, &String::from_str(&env, "X"));
    assert_eq!(env.auths()[0].0, a);
}

#[test]
fn second_vote_is_rejected() {
    let (env, client) = setup();
    let a = Address::generate(&env);
    client.vote(&a, &String::from_str(&env, "A"));
    let res = client.try_vote(&a, &String::from_str(&env, "B"));
    assert_eq!(res, Err(Ok(soroban_sdk::Error::from_contract_error(VoteError::AlreadyVoted as u32))));
    assert_eq!(client.results().len(), 1);
}

#[test]
fn empty_project_is_rejected() {
    let (env, client) = setup();
    let a = Address::generate(&env);
    let res = client.try_vote(&a, &String::from_str(&env, ""));
    assert_eq!(res, Err(Ok(soroban_sdk::Error::from_contract_error(VoteError::BadProjectName as u32))));
}
