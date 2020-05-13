use anyhow::{anyhow, format_err, Context, Result};
use graphql_client::*;
use log::info;
use prettytable::*;
use serde::*;
use structopt::StructOpt;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug"
)]
struct RepoView;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(name = "repository")]
    repo: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), anyhow::Error> {
    match repo_name.split('/').take(2).collect::<Vec<&str>>()[..] {
        [owner, name] => return Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

pub fn request() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: Env = envy::from_env().context("while reading from environment")?;

    let args = Command::from_args();

    let repo = args.repo;
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let q = RepoView::build_query(repo_view::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(config.github_api_token)
        .json(&q)
        .send()?;

    let Response { data, errors }: Response<repo_view::ResponseData> = res.json()?;
    info!("{:?}", data);

    match errors {
        Some(errors) => Err(anyhow!(
            "Error: {}",
            errors
                .iter()
                .fold(String::new(), |acc, err| acc + &err.message)
        )),
        None => Ok(()),
    }?;
    let response_data: repo_view::ResponseData = data.expect("Unexpected: missing response data");

    let stars: Option<i64> = response_data
        .repository
        .as_ref()
        .map(|repo| repo.stargazers.total_count);

    println!("{}/{} - 🌟 {}", owner, name, stars.unwrap_or(0),);
    let repository = response_data.repository.expect("missing repository");

    let mut issue_table = prettytable::Table::new();
    issue_table.add_row(row!(b => "issue", "comments"));
    for issue in &repository.issues.nodes.unwrap_or(vec![]) {
        if let Some(issue) = issue {
            issue_table.add_row(row!(issue.title, issue.comments.total_count));
        }
    }
    issue_table.printstd();

    let mut pull_request_table = prettytable::Table::new();
    pull_request_table.add_row(row!(b => "pull requests","comments", "commits","author"));

    let unknown_name = "unknown";

    if let Some(pull_requests) = repository.pull_requests.nodes {
        for pull_request in pull_requests {
            if let Some(pull_request) = pull_request {
                let author = pull_request.author.unwrap();
                let name = match author.on {
                    repo_view::RepoViewRepositoryPullRequestsNodesAuthorOn::User(user) => {
                        user.name.unwrap_or_else(|| unknown_name.to_string())
                    }
                    _ => unknown_name.to_string(),
                };
                pull_request_table.add_row(row!(
                    pull_request.title,
                    pull_request.comments.total_count,
                    pull_request.commits.total_count,
                    name
                ));
            }
        }
    }
    pull_request_table.printstd();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_name_works() {
        assert_eq!(
            parse_repo_name("github_owner/github_repo").unwrap(),
            ("github_owner", "github_repo")
        );
        assert!(parse_repo_name("abcd").is_err());
    }
}
