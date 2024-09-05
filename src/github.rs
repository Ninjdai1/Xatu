use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDate, Utc};
use octocrab::models::{self, issues::Issue, pulls::PullRequest, IssueState, Label};

#[derive(Clone, Debug)]
struct ParsedIssue {
    user: String,
    title: String,
    state: IssueState,
    creation_date: DateTime<Utc>,
    updated_date: DateTime<Utc>,
    closed_date: Option<DateTime<Utc>>,
    labels: Vec<Label>
}

#[derive(Debug, Clone, PartialEq)]
enum PRState {
    Open,
    Draft,
    Merged,
    Cancelled
}
impl Default for PRState {
    fn default() -> Self {
        Self::Open
    }
}

#[derive(Clone, Debug)]
struct ParsedPR {
    user: String,
    title: String,
    state: PRState,
    open_state: IssueState,
    creation_date: DateTime<Utc>,
    updated_date: DateTime<Utc>,
    closed_date: Option<DateTime<Utc>>,
    labels: Vec<Label>
}

#[derive(Copy, Clone, Debug)]
struct TimedStats {
    date: Option<NaiveDate>,
    opened_prs: usize,
    merged_prs: usize,
    cancelled_prs: usize,

    opened_issues: usize,
    closed_issues: usize,
}

impl Default for TimedStats {
    fn default() -> Self {
        Self { date: Some(chrono::offset::Utc::now().date_naive()), opened_prs: 0, merged_prs: 0, cancelled_prs: 0, opened_issues: 0, closed_issues: 0 }
    }
}

impl TimedStats {
    fn since_date(datetime: NaiveDate, issues: &mut [ParsedIssue], pull_requests: &mut [ParsedPR]) -> Self {
        Self {
            date: Some(datetime),
            opened_prs: pull_requests.iter().filter(|p| p.creation_date.date_naive() >= datetime).count(),
            merged_prs: pull_requests.iter().filter(|p| (p.state==PRState::Merged) && p.closed_date.is_some() && (p.closed_date.unwrap().date_naive() >= datetime)).count(),
            cancelled_prs: pull_requests.iter().filter(|p| (p.state==PRState::Cancelled) && p.closed_date.is_some() && (p.closed_date.unwrap().date_naive() >= datetime)).count(),
            opened_issues: issues.iter().filter(|i| i.creation_date.date_naive() >= datetime).count(),
            closed_issues: issues.iter().filter(|i| i.closed_date.is_some() && (i.closed_date.unwrap().date_naive() >= datetime)).count()
        }
    }

    fn on_date(datetime: NaiveDate, issues: &mut [ParsedIssue], pull_requests: &mut [ParsedPR]) -> Self {
        Self {
            date: Some(datetime),
            opened_prs: pull_requests.iter().filter(|p| p.creation_date.date_naive() == datetime).count(),
            merged_prs: pull_requests.iter().filter(|p| (p.state==PRState::Merged) && p.closed_date.is_some() && (p.closed_date.unwrap().date_naive() == datetime)).count(),
            cancelled_prs: pull_requests.iter().filter(|p| (p.state==PRState::Cancelled) && p.closed_date.is_some() && (p.closed_date.unwrap().date_naive() == datetime)).count(),
            opened_issues: issues.iter().filter(|i| i.creation_date.date_naive() == datetime).count(),
            closed_issues: issues.iter().filter(|i| i.closed_date.is_some() && (i.closed_date.unwrap().date_naive() == datetime)).count()
        }
    }
    
    fn all_time(issues: &mut [ParsedIssue], pull_requests: &mut [ParsedPR]) -> Self {
        Self {
            date: None,
            opened_prs: pull_requests.len(),
            merged_prs: pull_requests.iter().filter(|p| (p.state==PRState::Merged) && p.closed_date.is_some()).count(),
            cancelled_prs: pull_requests.iter().filter(|p| (p.state==PRState::Cancelled) && p.closed_date.is_some()).count(),
            opened_issues: issues.len(),
            closed_issues: issues.iter().filter(|i| i.closed_date.is_some()).count()
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct GithubData {
    date: DateTime<Utc>,
    open_issues: usize,
    confirmed_issues: usize,
    unconfirmed_issues: usize,
    feature_requests: usize,

    open_pull_requests: usize,
    ready_pull_requests: usize,
    draft_pull_requests: usize,

    stale_issues: Vec<ParsedIssue>,
    stale_pull_requests: Vec<ParsedPR>,

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

        let _today = self.date.date_naive();
        let yesterday = self.date.date_naive().pred_opt().unwrap();
        let mut last_7_days = yesterday;
        for _ in 0..7 {last_7_days = last_7_days.pred_opt().unwrap()};
        let mut last_30_days = yesterday;
        for _ in 0..30 {last_30_days = last_30_days.pred_opt().unwrap()};
        let mut last_365_days = yesterday;
        for _ in 0..365 { last_365_days = last_365_days.pred_opt().unwrap()};

        let mut issues: Vec<ParsedIssue> = Vec::new();
        let mut pull_requests: Vec<ParsedPR> = Vec::new();


        let octocrab = octocrab::instance();
        let mut issues_page = octocrab
            .issues("rh-hideout", "pokeemerald-expansion")
            .list()
            .state(octocrab::params::State::All)
            .per_page(100)
            .send()
            .await.unwrap();
        loop {
            for issue in &issues_page {
                if issue.pull_request.is_none(){issues.push(parse_issue(issue.clone()))}
            }
            issues_page = match octocrab
                .get_page::<models::issues::Issue>(&issues_page.next)
                .await.unwrap()
            {
                Some(next_page) => next_page,
                None => break,
            }
        }

        let mut pull_requests_page = octocrab
            .pulls("rh-hideout", "pokeemerald-expansion")
            .list()
            .state(octocrab::params::State::All)
            .per_page(100)
            .send()
            .await.unwrap();
        loop {
            for pr in &pull_requests_page {
                pull_requests.push(parse_pr(pr.clone()));
            }
            pull_requests_page = match octocrab
                .get_page::<models::pulls::PullRequest>(&pull_requests_page.next)
                .await.unwrap()
            {
                Some(next_page) => next_page,
                None => break,
            }
        }
        
        for issue in issues.iter().filter(|i| i.state==IssueState::Open) {
            if issue.labels.iter().any(|l| l.name == "status: unconfirmed") {self.unconfirmed_issues += 1;}
            else if issue.labels.iter().any(|l| l.name == "status: confirmed") {self.confirmed_issues += 1;}
            else if issue.labels.iter().any(|l| l.name == "feature-request") {self.feature_requests += 1;}
        }
        self.open_issues = self.confirmed_issues+self.unconfirmed_issues+self.feature_requests;
        
        self.draft_pull_requests = pull_requests.iter().filter(|p| (p.open_state==IssueState::Open) & (p.state==PRState::Draft)).count();
        self.ready_pull_requests = pull_requests.iter().filter(|p| p.state==PRState::Open).count();
        self.open_pull_requests = self.draft_pull_requests + self.ready_pull_requests;

        self.yesterday = TimedStats::on_date(yesterday, &mut issues, &mut pull_requests);
        self.last_week = TimedStats::since_date(last_7_days, &mut issues, &mut pull_requests);
        self.last_month = TimedStats::since_date(last_30_days, &mut issues, &mut pull_requests);
        self.last_year = TimedStats::since_date(last_365_days, &mut issues, &mut pull_requests);
        self.all = TimedStats::all_time(&mut issues, &mut pull_requests);

        let test = octocrab::instance().ratelimit().get().await.unwrap();

        println!("Rate limit: {:#?}", test.resources.core);
        println!("Resets in {:#?} minutes", (test.resources.core.reset - SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())/60);
    }

    pub fn render(&self) -> String {
        println!("{:#?}", self);
        let mut md = String::from("# Raw Stats\n\n");
        md.push_str(&format!("{} open issues ({} confirmed / {} unconfirmed / {} feature requests)\n\n", self.open_issues, self.confirmed_issues, self.unconfirmed_issues, self.feature_requests));
        md.push_str(&format!("{} open PRs ({} ready for merge / {} draft)\n", self.open_pull_requests, self.ready_pull_requests, self.draft_pull_requests));

        md.push_str("# Stats\n\nAll stats are displayed as:\n\n**Metric**: yesterday | last 7 days | last 30 days | last 365 days | all time.\n\n");

        let yesterday_date_span = format!("{}", self.yesterday.date.unwrap());
        let last_7_days_date_span = format!("{l7}..{y}", y=self.yesterday.date.unwrap(), l7=self.last_week.date.unwrap());
        let last_30_days_date_span = format!("{l30}..{y}", y=self.yesterday.date.unwrap(), l30=self.last_month.date.unwrap());
        let last_365_days_date_span = format!("{l365}..{y}", y=self.yesterday.date.unwrap(), l365=self.last_year.date.unwrap());

        md.push_str(&format!(
                "## Pull Requests\n\n**Opened PRs**: [{py}]({PR_OPENED}{CREATED_STRING}{yesterday_date_span}) | [{p7}]({PR_OPENED}{CREATED_STRING}{last_7_days_date_span}) | [{p30}]({PR_OPENED}{CREATED_STRING}{last_30_days_date_span}) | [{p365}]({PR_OPENED}{CREATED_STRING}{last_365_days_date_span}) | [{pa}]({PR_OPENED})\n\n",
            py=self.yesterday.opened_prs,
            p7=self.last_week.opened_prs,
            p30=self.last_month.opened_prs,
            p365=self.last_year.opened_prs,
            pa=self.all.opened_prs
        ));
        md.push_str(&format!(
                "**Merged PRs**: [{py}]({PR_MERGED}{MERGED_STRING}{yesterday_date_span}) | [{p7}]({PR_MERGED}{MERGED_STRING}{last_7_days_date_span}) | [{p30}]({PR_MERGED}{MERGED_STRING}{last_30_days_date_span}) | [{p365}]({PR_MERGED}{MERGED_STRING}{last_365_days_date_span}) | [{pa}]({PR_MERGED})\n\n",
            py=self.yesterday.merged_prs,
            p7=self.last_week.merged_prs,
            p30=self.last_month.merged_prs,
            p365=self.last_year.merged_prs,
            pa=self.all.merged_prs
        ));

        md.push_str(&format!("**Merge Rate**: {:.2} | {:.2} | {:.2} | {:.2} | {:.2}\n\n",
                (self.yesterday.merged_prs as f64)/(self.yesterday.opened_prs as f64),
                (self.last_week.merged_prs as f64)/(self.last_week.opened_prs as f64),
                (self.last_month.merged_prs as f64)/(self.last_month.opened_prs as f64),
                (self.last_year.merged_prs as f64)/(self.last_year.opened_prs as f64),
                (self.all.merged_prs as f64)/(self.all.opened_prs as f64),
                ));
        md.push_str(&format!("**PR Growth**: {} | {} | {} | {} | {}\n\n",
                self.yesterday.opened_prs as i64 - self.yesterday.merged_prs as i64,
                self.last_week.opened_prs as i64 - self.last_week.merged_prs as i64,
                self.last_month.opened_prs as i64 - self.last_month.merged_prs as i64,
                self.last_year.opened_prs as i64 - self.last_year.merged_prs as i64,
                self.all.opened_prs as i64 - self.all.merged_prs as i64,
                ));

        md.push_str(&format!(
                "## Issues\n\n**Opened Issues**: [{py}]({ISSUE_OPENED}{CREATED_STRING}{yesterday_date_span}) | [{p7}]({ISSUE_OPENED}{CREATED_STRING}{last_7_days_date_span}) | [{p30}]({ISSUE_OPENED}{CREATED_STRING}{last_30_days_date_span}) | [{p365}]({ISSUE_OPENED}{CREATED_STRING}{last_365_days_date_span}) | [{pa}]({ISSUE_OPENED})\n\n",
            py=self.yesterday.opened_issues,
            p7=self.last_week.opened_issues,
            p30=self.last_month.opened_issues,
            p365=self.last_year.opened_issues,
            pa=self.all.opened_issues
        ));
        md.push_str(&format!(
                "**Closed Issues**: [{py}]({ISSUE_CLOSED}{CLOSED_STRING}{yesterday_date_span}) | [{p7}]({ISSUE_CLOSED}{CLOSED_STRING}{last_7_days_date_span}) | [{p30}]({ISSUE_CLOSED}{CLOSED_STRING}{last_30_days_date_span}) | [{p365}]({ISSUE_CLOSED}{CLOSED_STRING}{last_365_days_date_span}) | [{pa}]({ISSUE_CLOSED})\n\n",
            py=self.yesterday.opened_issues,
            p7=self.last_week.opened_issues,
            p30=self.last_month.opened_issues,
            p365=self.last_year.opened_issues,
            pa=self.all.opened_issues
        ));

        md.push_str(&format!("**Resolution Rate**: {:.2} | {:.2} | {:.2} | {:.2} | {:.2}\n\n",
                (self.yesterday.closed_issues as f64)/(self.yesterday.opened_issues as f64),
                (self.last_week.closed_issues as f64)/(self.last_week.opened_issues as f64),
                (self.last_month.closed_issues as f64)/(self.last_month.opened_issues as f64),
                (self.last_year.closed_issues as f64)/(self.last_year.opened_issues as f64),
                (self.all.closed_issues as f64)/(self.all.opened_issues as f64),
            ));
        md.push_str(&format!("**Issue Growth**: {} | {} | {} | {} | {}\n\n",
                self.yesterday.opened_issues as i64 - self.yesterday.closed_issues as i64,
                self.last_week.opened_issues as i64 - self.last_week.closed_issues as i64,
                self.last_month.opened_issues as i64 - self.last_month.closed_issues as i64,
                self.last_year.opened_issues as i64 - self.last_year.closed_issues as i64,
                self.all.opened_issues as i64 - self.all.closed_issues as i64,
            ));

        md
    }
}

static CREATED_STRING: &str = "+created%3A";
static MERGED_STRING: &str = "+merged%3A";
static CLOSED_STRING: &str = "+closed%3A";
static PR_OPENED: &str = "https://github.com/rh-hideout/pokeemerald-expansion/pulls?q=is%3Apr+sort%3Aupdated-asc";
static PR_MERGED: &str = "https://github.com/rh-hideout/pokeemerald-expansion/pulls?q=is%3Apr+is%3Amerged+sort%3Aupdated-asc+draft%3Afalse";
static ISSUE_OPENED: &str = "https://github.com/rh-hideout/pokeemerald-expansion/issues?q=is%253Aissue+sort%3Aupdated-asc";
static ISSUE_CLOSED: &str = "https://github.com/rh-hideout/pokeemerald-expansion/issues?q=is%253Aissue+is%253Aclosed+sort%3Aupdated-asc";

fn parse_issue(issue: Issue) -> ParsedIssue {
    ParsedIssue {
        user: issue.user.login.clone(),
        title: issue.title.clone(),
        state: issue.state,
        creation_date: issue.created_at,
        updated_date: issue.updated_at,
        closed_date: issue.closed_at,
        labels: issue.labels.clone()
    }
}

fn parse_pr(pr: PullRequest) -> ParsedPR {
    //println!("{:#?}", pr);
    ParsedPR {
        user: pr.user.expect("Failed getting pr user").login,
        title: pr.title.expect("Failed getting pr title"),
        state: match pr.draft {
            Some(true) => PRState::Draft,
            _ => match pr.merged_at {
                Some(_) => PRState::Merged,
                _ => match pr.closed_at {
                    Some(_) => PRState::Cancelled,
                    _=> PRState::Open
                }
            }
        },
        open_state: pr.state.expect("Failed getting pr state"),
        creation_date: pr.created_at.expect("Failed getting pr creation date"),
        updated_date: pr.updated_at.expect("Failed getting pr update date"),
        closed_date: pr.closed_at,
        labels: pr.labels.expect("Failed getting pr labels")
    }
}
