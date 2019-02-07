extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
use reqwest::Client;
use std::fs::{OpenOptions};
use std::fs;
use std::io::copy;
extern crate rand;
use std::thread;
use std::process::Command;
use std::fmt;

#[derive(Debug)]
struct Epic(String);
impl fmt::Display for Epic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for Epic {}


fn main() {
    loop {
        let client = Client::new();
        let gid = match get_gid(client.clone()) {
            Ok(gid) => {
                gid
            }
            Err(e) => {
                println!("{}", e);
                continue
            }
        };
        match solve_captcha(gid, client) {
            Ok(captcha) => {
                captcha
            }
            Err(e) => {
                println!("{}", e);
                continue
            }
        };
    }
}

fn solve_captcha(gid: String, client: reqwest::Client) -> Result<(), Box<std::error::Error>> {
    let folder = randomize();
    fs::create_dir(format!("./data/{}", folder.clone(

    )))?;    
    println!("Downloading captcha gid: {}", gid);
    let dir = format!("./data/{}/{}.png", folder.clone(), gid);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dir.clone())?;
    let target = format!("https://store.steampowered.com/login/rendercaptcha?gid={}", gid);
    let mut response = client.get(&target).send()?;
    copy(&mut response, &mut file)?;

    //sip
    Command::new("python")
        .args(&["captcha_recognize.py", "--captcha_dir", &dir])
        .output()?;
    //sip

        
    /*
    Ignore this, I was testing stuff.
    use std::io::Write;
    print!("Captcha solution: ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let nowline = input.replace("\n", "");
    println!("{:?}", nowline);
    */

    let ans = fs::read_to_string("./ans.txt")?;
    let array: Vec<&str> = ans.split("\n").collect();
    let mut solution: String = "a".to_string();

    let mut i = 0;
    for sol in array.clone() {
        i+=1;
        match check_captcha(gid.clone(), sol.to_string(), client.clone()) {
            Ok(nice) => {
                if nice == true {
                    solution = sol.to_string();
                    break;
                } else {
                    if i == array.len() {
                        let err = format!("None of the solutions are correct so have a nice day.");
                        fs::remove_dir_all(folder.clone())?;
                        return Err(Box::new(Epic(err.into())));
                    }
                    continue;
                }
            }
            Err(err) => {
                return Err(err);
            }
        };
    }

    //Part where it downloads all of them

    let mut children = vec![];
    for _i in 0..100 {
        let gid = gid.clone();
        let solution = solution.clone();
        let client = client.clone();
        let folder = folder.clone();
        children.push(
        thread::spawn(move || {
        println!("Downloading captcha gid: {}", gid);
        let random = randomize();
        let dir = format!("./data/{}/{}_{}.png", folder, solution, random);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dir)
            .unwrap();
        let target = format!("https://store.steampowered.com/login/rendercaptcha?gid={}", gid);
        let mut response = client.get(&target).send().unwrap();
        copy(&mut response, &mut file).unwrap();
        }))
    }
    for child in children {
        match child.join() {
            Err(e) => {
                println!("{:?}",e)
            }
            Ok(_a) => {
            }
        }
    }
    let random = randomize();
    fs::rename(dir, format!("./data/{}/{}_{}.png", folder, solution, random))?;
    Ok(())
}

fn get_gid(client: reqwest::Client) -> Result<String, reqwest::Error> {
    #[derive(Deserialize)]
    struct Gid {
        gid: String,
    }
    let mut response = client.get("https://store.steampowered.com/join/refreshcaptcha/").send()?;
    let res: Gid = response.json()?;
    Ok(res.gid)
}

fn check_captcha(gid: String, sol: String, client: reqwest::Client) -> Result<bool, Box<std::error::Error>> {
    #[derive(Deserialize)]
    struct Valid {
        bCaptchaMatches: bool,
    }
    let params = [("captchagid", gid), ("captcha_text", sol), ("email", "bencode07@illegal.loli.su".to_string())];
    let mut verify = client.post("https://store.steampowered.com/join/verifycaptcha/")
        .form(&params)
        .send()?;
    let verification: Valid = verify.json()?;

    Ok(verification.bCaptchaMatches)
}

fn randomize() -> String {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string
}
