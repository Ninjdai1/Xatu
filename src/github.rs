use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}, u64};

use chrono::{DateTime, Utc};
use octocrab::{params, Octocrab};

#[derive(Copy, Clone, Default, Debug)]
struct TimedStats {
    opened_prs: u64,
    merged_prs: u64,
    cancelled_prs: u64,

    opened_issues: u64,
    closed_issues: u64,
}

impl TimedStats {
    pub async fn all(octocrab: &Arc<Octocrab>) -> Self {
        let opened_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr"))
            .per_page(1)
            .send()
            .await.unwrap();
        let cancelled_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr is:closed is:unmerged"))
            .per_page(1)
            .send()
            .await.unwrap();
        let merged_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr is:merged"))
            .per_page(1)
            .send()
            .await.unwrap();

        let opened_issues = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:issue is:open"))
            .per_page(1)
            .send()
            .await.unwrap();
        let closed_issues = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:issue is:closed"))
            .per_page(1)
            .send()
            .await.unwrap();
        Self {
            opened_prs: opened_prs.total_count.unwrap(),
            cancelled_prs: cancelled_prs.total_count.unwrap(),
            merged_prs: merged_prs.total_count.unwrap(),
            opened_issues: opened_issues.total_count.unwrap(),
            closed_issues: closed_issues.total_count.unwrap(),
        }
    }
    pub async fn from_datestring(octocrab: &Arc<Octocrab>, datestring: String) -> Self {
        let opened_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr created:{datestring}"))
            .per_page(1)
            .send()
            .await.unwrap();
        let cancelled_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr closed:{datestring} is:unmerged"))
            .per_page(1)
            .send()
            .await.unwrap();
        let merged_prs = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:pr merged:{datestring}"))
            .per_page(1)
            .send()
            .await.unwrap();

        let opened_issues = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:issue created:{datestring}"))
            .per_page(1)
            .send()
            .await.unwrap();
        let closed_issues = octocrab
            .search()
            .issues_and_pull_requests(&format!("repo:rh-hideout/pokeemerald-expansion is:issue closed:{datestring}"))
            .per_page(1)
            .send()
            .await.unwrap();
        Self {
            opened_prs: opened_prs.total_count.unwrap(),
            cancelled_prs: cancelled_prs.total_count.unwrap(),
            merged_prs: merged_prs.total_count.unwrap(),
            opened_issues: opened_issues.total_count.unwrap(),
            closed_issues: closed_issues.total_count.unwrap(),
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct GithubData {
    date: DateTime<Utc>,
    issues: u64,
    confirmed_issues: u64,
    unconfirmed_issues: u64,

    pull_requests: u64,
    ready_pull_requests: u64,
    draft_pull_requests: u64,

    yesterday: TimedStats,
    last_week: TimedStats,
    last_month: TimedStats,
    last_year: TimedStats,
    all: TimedStats,
}

impl GithubData {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn fetch(&mut self) {
        self.date = chrono::offset::Utc::now();

        let today = self.date.date_naive();
        println!("Today: {:?}", today);
        let yesterday = self.date.date_naive().pred_opt().unwrap();

        println!("Yesterday: {:?}", yesterday);
        
        let mut last_7_days = yesterday.clone();
        for _ in 0..7 {
            last_7_days = last_7_days.pred_opt().unwrap()
        };
        println!("Last 7 days: {:?}", last_7_days);

        let mut last_30_days = yesterday.clone();
        for _ in 0..30 {
            last_30_days = last_30_days.pred_opt().unwrap()
        };
        println!("Last 30 days: {:?}", last_30_days);

        let mut last_365_days = yesterday.clone();
        for _ in 0..365 {
            last_365_days = last_365_days.pred_opt().unwrap()
        };
        println!("Last 365 days: {:?}", last_365_days);

        let octocrab = octocrab::instance();
        
        let unconfirmed_issues_page = octocrab
            .search()
            .issues_and_pull_requests("repo:rh-hideout/pokeemerald-expansion is:issue label:\"status: unconfirmed\" is:open")
            .per_page(1)
            .send()
            .await.unwrap();
        
        let confirmed_issues_page = octocrab
            .search()
            .issues_and_pull_requests("repo:rh-hideout/pokeemerald-expansion is:issue label:\"status: confirmed\" is:open")
            .per_page(1)
            .send()
            .await.unwrap();

        let draft_prs_page = octocrab
            .search()
            .issues_and_pull_requests("repo:rh-hideout/pokeemerald-expansion is:open draft:true")
            .per_page(1)
            .send()
            .await.unwrap();

        let ready_prs_page = octocrab
            .search()
            .issues_and_pull_requests("repo:rh-hideout/pokeemerald-expansion is:open draft:false")
            .per_page(1)
            .send()
            .await.unwrap();

        self.unconfirmed_issues = unconfirmed_issues_page.total_count.unwrap();
        self.confirmed_issues = confirmed_issues_page.total_count.unwrap();
        self.issues = self.unconfirmed_issues + self.confirmed_issues;

        self.draft_pull_requests = draft_prs_page.total_count.unwrap();
        self.ready_pull_requests = ready_prs_page.total_count.unwrap();
        self.pull_requests = self.draft_pull_requests + self.ready_pull_requests;

        self.yesterday = TimedStats::from_datestring(&octocrab, format!("{yesterday}")).await;
        self.last_week = TimedStats::from_datestring(&octocrab, format!(">{last_7_days}")).await;
        self.last_month = TimedStats::from_datestring(&octocrab, format!(">{last_30_days}>")).await;
        self.last_year = TimedStats::from_datestring(&octocrab, format!(">{last_365_days}")).await;

        self.all = TimedStats::all(&octocrab).await;
            
        let test = octocrab::instance().ratelimit().get().await.unwrap();

        println!("Search Ratelimit: {:#?}", test.resources.search);
        println!("Resets in {:#?} seconds", test.resources.search.reset - SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    }

    pub fn render(&self) -> String {
        println!("{:#?}", self);
        let mut md = String::from("# Raw Stats\n\n");
        md.push_str(&format!(""));

        md
    }
}
