#![no_std]
//! Commit Club Vote: one wallet, one vote, for the best Commit Club project.
//!
//! Each Stellar Testnet wallet can vote once for a project (by name). Anyone
//! can read the tally, so the ranking is public and verifiable on-chain.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address, Env,
    String, Vec,
};

const DAY_IN_LEDGERS: u32 = 17_280;
const BUMP_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;
const BUMP_TO: u32 = 30 * DAY_IN_LEDGERS;
const MAX_PROJECTS: u32 = 100;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VoteError {
    AlreadyVoted = 1,
    BadProjectName = 2,
    TooManyProjects = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tally {
    pub project: String,
    pub votes: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// All project names that have at least one vote, in first-vote order.
    Projects,
    /// Votes for one project.
    Votes(String),
    /// What a wallet voted for.
    Voter(Address),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Voted {
    #[topic]
    pub voter: Address,
    pub project: String,
}

#[contract]
pub struct Vote;

#[contractimpl]
impl Vote {
    /// Cast this wallet's single vote for `project`. Returns the project's new total.
    pub fn vote(env: Env, voter: Address, project: String) -> u32 {
        voter.require_auth();
        if project.len() == 0 || project.len() > 64 {
            panic_with_error!(&env, VoteError::BadProjectName);
        }
        let voter_key = DataKey::Voter(voter.clone());
        if env.storage().persistent().has(&voter_key) {
            panic_with_error!(&env, VoteError::AlreadyVoted);
        }

        let votes_key = DataKey::Votes(project.clone());
        let votes: u32 = env.storage().persistent().get(&votes_key).unwrap_or(0) + 1;
        if votes == 1 {
            let mut projects: Vec<String> =
                env.storage().instance().get(&DataKey::Projects).unwrap_or(Vec::new(&env));
            if projects.len() >= MAX_PROJECTS {
                panic_with_error!(&env, VoteError::TooManyProjects);
            }
            projects.push_back(project.clone());
            env.storage().instance().set(&DataKey::Projects, &projects);
        }

        let p = env.storage().persistent();
        p.set(&votes_key, &votes);
        p.extend_ttl(&votes_key, BUMP_THRESHOLD, BUMP_TO);
        p.set(&voter_key, &project);
        p.extend_ttl(&voter_key, BUMP_THRESHOLD, BUMP_TO);
        env.storage().instance().extend_ttl(BUMP_THRESHOLD, BUMP_TO);

        Voted { voter, project }.publish(&env);
        votes
    }

    /// Every project with its vote count (unsorted; the page sorts it).
    pub fn results(env: Env) -> Vec<Tally> {
        let projects: Vec<String> =
            env.storage().instance().get(&DataKey::Projects).unwrap_or(Vec::new(&env));
        let mut out = Vec::new(&env);
        for project in projects.iter() {
            let votes = env.storage().persistent().get(&DataKey::Votes(project.clone())).unwrap_or(0);
            out.push_back(Tally { project, votes });
        }
        out
    }

    /// The project a wallet voted for, if any.
    pub fn voted_for(env: Env, voter: Address) -> Option<String> {
        env.storage().persistent().get(&DataKey::Voter(voter))
    }
}

mod test;
