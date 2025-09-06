use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use colored::*;
use std::env;
use terminal_size::{Width, Height, terminal_size};

#[derive(Parser)]
#[command(name = "gitstat")]
#[command(about = "Display GitHub activity schema for any user")]
struct Args {
    /// GitHub username
    username: String,
    
    /// GitHub access token (or use GITHUB_TOKEN environment variable)
    #[arg(short, long)]
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    name: Option<String>,
    public_repos: u32,
    followers: u32,
    following: u32,
}

// Structures pour la requête GraphQL
#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: GraphQLVariables,
}

#[derive(Serialize)]
struct GraphQLVariables {
    username: String,
}

// Structures pour la réponse GraphQL
#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GraphQLData {
    user: Option<GitHubUserWithContributions>,
}

#[derive(Debug, Deserialize)]
struct GitHubUserWithContributions {
    #[allow(dead_code)]
    login: String,
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "contributionsCollection")]
    contributions_collection: ContributionsCollection,
}

#[derive(Debug, Deserialize)]
struct ContributionsCollection {
    #[serde(rename = "contributionCalendar")]
    contribution_calendar: ContributionCalendar,
}

#[derive(Debug, Deserialize)]
struct ContributionCalendar {
    #[serde(rename = "totalContributions")]
    total_contributions: u32,
    weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Deserialize)]
struct ContributionWeek {
    #[serde(rename = "contributionDays")]
    contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Deserialize, Clone)]
struct ContributionDay {
    #[allow(dead_code)]
    date: String,
    #[serde(rename = "contributionCount")]
    contribution_count: u32,
    #[allow(dead_code)]
    color: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Get token from arguments or environment variables
    let token = args.token.or_else(|| env::var("GITHUB_TOKEN").ok());
    
    if token.is_none() {
        eprintln!("Error: GitHub token required!");
        eprintln!("You can:");
        eprintln!("   1. Pass token with --token YOUR_TOKEN");
        eprintln!("   2. Set GITHUB_TOKEN environment variable");
        eprintln!("   3. Create a token at: https://github.com/settings/tokens");
        eprintln!("      (Required permissions: 'read:user' only)");
        std::process::exit(1);
    }
    
    let token = token.unwrap();
    
    let client = Client::new();
    
    // Get basic user information
    match get_user_info(&client, &args.username).await {
        Ok(user) => {
            // Get and display real contributions
            match get_user_contributions_real(&client, &args.username, &token).await {
                Ok(contributions) => {
                    display_user_profile(&user, &contributions);
                }
                Err(e) => {
                    eprintln!("Error retrieving contributions: {}", e);
                    eprintln!("Please verify your token is valid and has proper permissions");
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn get_user_info(client: &Client, username: &str) -> Result<GitHubUser, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/users/{}", username);
    let response = client
        .get(&url)
        .header("User-Agent", "gitstat-cli")
        .send()
        .await?;
    
    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(format!("User '{}' not found", username).into())
    }
}

async fn get_user_contributions_real(
    client: &Client,
    username: &str,
    token: &str,
) -> Result<ContributionCalendar, Box<dyn std::error::Error>> {
    let query = r#"
        query($username: String!) {
            user(login: $username) {
                login
                name
                contributionsCollection {
                    contributionCalendar {
                        totalContributions
                        weeks {
                            contributionDays {
                                date
                                contributionCount
                                color
                            }
                        }
                    }
                }
            }
        }
    "#;
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: GraphQLVariables {
            username: username.to_string(),
        },
    };
    
    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "gitstat-cli")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let graphql_response: GraphQLResponse = response.json().await?;
    
    if let Some(errors) = graphql_response.errors {
        let error_messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
        return Err(format!("GraphQL errors: {}", error_messages.join(", ")).into());
    }
    
    let data = graphql_response
        .data
        .ok_or("No data returned by API")?;
        
    let user = data
        .user
        .ok_or(format!("User '{}' not found", username))?;
    
    Ok(user.contributions_collection.contribution_calendar)
}

fn display_user_profile(user: &GitHubUser, calendar: &ContributionCalendar) {
    let (term_width, _) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w as usize, h as usize)
    } else {
        (80, 24) 
    };
    
    // Calculate available space for the calendar 
    let calendar_width = std::cmp::min(53, term_width.saturating_sub(40) / 2);
    
    draw_header(user, term_width);
    draw_contribution_calendar(calendar, calendar_width, term_width);
    draw_statistics(calendar, term_width);
}

fn draw_header(user: &GitHubUser, term_width: usize) {
    let title = format!(" {} ", user.login);
    let padding = (term_width.saturating_sub(title.len())) / 2;
    
    // Top border
    println!("{}", "─".repeat(term_width).bright_blue());
    
    // Title line
    println!("{}{}{}", 
        " ".repeat(padding), 
        title.bright_white().bold(),
        " ".repeat(term_width.saturating_sub(padding + title.len()))
    );
    
    // User info section
    let name = user.name.as_ref().unwrap_or(&user.login);
    let info_line = format!("Name: {}  |  Repos: {}  |  Followers: {}  |  Following: {}", 
        name, user.public_repos, user.followers, user.following);
    
    let info_padding = (term_width.saturating_sub(info_line.len())) / 2;
    println!("{}{}", 
        " ".repeat(info_padding),
        info_line.bright_cyan()
    );
    
    println!("{}", "─".repeat(term_width).bright_blue());
}

fn draw_contribution_calendar(calendar: &ContributionCalendar, calendar_width: usize, term_width: usize) {
    let title = " GitHub Activity (Last Year) ";
    let title_padding = (term_width.saturating_sub(title.len())) / 2;
    
    println!("{}{}", " ".repeat(title_padding), title.bright_white().bold());
    
    let total_text = format!("Total Contributions: {}", calendar.total_contributions);
    let total_padding = (term_width.saturating_sub(total_text.len())) / 2;
    println!("{}{}\n", " ".repeat(total_padding), total_text.bright_blue());
    
    // Month headers
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    
    let cal_padding = (term_width.saturating_sub(calendar_width + 8)) / 2;
    print!("{}", " ".repeat(cal_padding));
    print!("        ");
    
    let weeks_to_show = std::cmp::min(calendar.weeks.len(), calendar_width);
    for i in 0..weeks_to_show {
        if i % 4 == 0 && i / 4 < months.len() {
            print!("{}", months[i / 4].bright_blue());
        } else {
            print!(" ");
        }
    }
    println!();
    
    // Days of week labels
    let weekdays = ["Mon", "Wed", "Fri"];
    
    // Draw the calendar grid
    for row in 0..7 {
        print!("{}", " ".repeat(cal_padding));
        
        if row % 2 == 1 && row / 2 < weekdays.len() {
            print!("{:>3} ", weekdays[row / 2].bright_blue());
        } else {
            print!("    ");
        }
        
        for week_idx in 0..weeks_to_show {
            if week_idx < calendar.weeks.len() {
                let week = &calendar.weeks[week_idx];
                if let Some(day) = week.contribution_days.get(row) {
                    let symbol = match day.contribution_count {
                        0 => "■".truecolor(45, 51, 59),        
                        1..=2 => "■".truecolor(14, 68, 121),   
                        3..=5 => "■".truecolor(33, 110, 177),  
                        6..=10 => "■".truecolor(52, 152, 219), 
                        _ => "■".truecolor(116, 185, 255),     
                    };
                    print!("{}", symbol);
                } else {
                    print!(" ");
                }
            } else {
                print!(" ");
            }
        }
        println!();
    }
    
    // Legend with actual colors
    let legend_padding = (term_width.saturating_sub(35)) / 2;
    print!("\n{}   Less  ", " ".repeat(legend_padding));
    print!("{}", "■".truecolor(45, 51, 59));        
    print!("{}", "■".truecolor(14, 68, 121));      
    print!("{}", "■".truecolor(33, 110, 177));     
    print!("{}", "■".truecolor(52, 152, 219));      
    print!("{}", "■".truecolor(116, 185, 255));     
    println!("  More");
}

fn draw_statistics(calendar: &ContributionCalendar, term_width: usize) {
    let days_with_contributions = calendar.weeks.iter()
        .flat_map(|w| &w.contribution_days)
        .filter(|d| d.contribution_count > 0)
        .count();
    
    let max_contributions = calendar.weeks.iter()
        .flat_map(|w| &w.contribution_days)
        .map(|d| d.contribution_count)
        .max()
        .unwrap_or(0);
    
    let average = if days_with_contributions > 0 {
        calendar.total_contributions as f32 / days_with_contributions as f32
    } else {
        0.0
    };
    
    println!();
    let stats_title = " Statistics ";
    let stats_padding = (term_width.saturating_sub(stats_title.len())) / 2;
    println!("{}{}", " ".repeat(stats_padding), stats_title.bright_white().bold());
    
    let stats_line = format!("Active Days: {}  |  Max/Day: {}  |  Avg/Active Day: {:.1}", 
        days_with_contributions, max_contributions, average);
    let stats_line_padding = (term_width.saturating_sub(stats_line.len())) / 2;
    println!("{}{}", " ".repeat(stats_line_padding), stats_line.bright_cyan());
    
    // Bottom border
    println!("{}", "─".repeat(term_width).bright_blue());
}
