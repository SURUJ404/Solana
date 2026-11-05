use anchor_lang::prelude::*;

declare_id!("DzSG9vpCQttfKoLAYxCJHnkKcU1LZN3arLw6vDJ8vk5k");

const MAX_POLL_NAME_LEN: usize = 32;
const MAX_POLL_DESCRIPTION_LEN: usize = 280;
const MAX_CANDIDATE_NAME_LEN: usize = 32;

#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        _poll_id: u64,
        start_time: u64,
        end_time: u64,
        name: String,
        description: String,
    ) -> Result<()> {
        require!(
            name.len() <= MAX_POLL_NAME_LEN,
            ErrorCode::NameTooLong
        );
        require!(
            description.len() <= MAX_POLL_DESCRIPTION_LEN,
            ErrorCode::DescriptionTooLong
        );
        require!(end_time > start_time, ErrorCode::InvalidVotingWindow);

        let poll_account = &mut ctx.accounts.poll_account;
        poll_account.poll_authority = ctx.accounts.signer.key();
        poll_account.poll_name = name;
        poll_account.poll_description = description;
        poll_account.poll_voting_start = start_time;
        poll_account.poll_voting_end = end_time;
        Ok(())
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        _poll_id: u64,
        candidate: String,
    ) -> Result<()> {
        require!(
            candidate.len() <= MAX_CANDIDATE_NAME_LEN,
            ErrorCode::NameTooLong
        );

        let now = current_time()?;
        require!(
            now <= ctx.accounts.poll_account.poll_voting_end,
            ErrorCode::VotingEnded
        );

        ctx.accounts.candidate_account.candidate_name = candidate;
        let poll_account = &mut ctx.accounts.poll_account;
        poll_account.poll_option_index = poll_account
            .poll_option_index
            .checked_add(1)
            .ok_or(ErrorCode::CounterOverflow)?;
        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
        let now = current_time()?;
        let poll_account = &ctx.accounts.poll_account;

        require!(
            now > poll_account.poll_voting_start,
            ErrorCode::VotingNotStarted
        );
        require!(now <= poll_account.poll_voting_end, ErrorCode::VotingEnded);

        let candidate_account = &mut ctx.accounts.candidate_account;
        candidate_account.candidate_votes = candidate_account
            .candidate_votes
            .checked_add(1)
            .ok_or(ErrorCode::CounterOverflow)?;

        let receipt = &mut ctx.accounts.vote_receipt;
        receipt.poll = poll_account.key();
        receipt.candidate = candidate_account.key();
        receipt.voter = ctx.accounts.signer.key();
        receipt.bump = ctx.bumps.vote_receipt;

        Ok(())
    }
}

/// Read the cluster clock as an unsigned timestamp.
///
/// `unix_timestamp` is signed while poll windows are stored as `u64`; refusing
/// negative timestamps avoids sign-wrap turning a closed poll into an open one.
fn current_time() -> Result<u64> {
    let timestamp = Clock::get()?.unix_timestamp;
    require!(timestamp >= 0, ErrorCode::VotingNotStarted);
    Ok(timestamp as u64)
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
        constraint = poll_account.poll_authority == signer.key() @ ErrorCode::Unauthorized,
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [b"candidate".as_ref(), poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        mut,
        seeds = [b"candidate".as_ref(), poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + VoteReceipt::INIT_SPACE,
        seeds = [b"vote".as_ref(), poll_account.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub vote_receipt: Account<'info, VoteReceipt>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    pub poll_authority: Pubkey,
    #[max_len(32)]
    pub poll_name: String,
    #[max_len(280)]
    pub poll_description: String,
    pub poll_voting_start: u64,
    pub poll_voting_end: u64,
    pub poll_option_index: u64,
}

/// One receipt per voter per poll, preventing repeat voting.
#[account]
#[derive(InitSpace)]
pub struct VoteReceipt {
    pub poll: Pubkey,
    pub candidate: Pubkey,
    pub voter: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Voting has not started yet")]
    VotingNotStarted,
    #[msg("Voting has ended")]
    VotingEnded,
    #[msg("Only the poll authority can add candidates")]
    Unauthorized,
    #[msg("Voting end time must be after the start time")]
    InvalidVotingWindow,
    #[msg("Name exceeds the maximum supported length")]
    NameTooLong,
    #[msg("Description exceeds the maximum supported length")]
    DescriptionTooLong,
    #[msg("Counter overflow")]
    CounterOverflow,
}
