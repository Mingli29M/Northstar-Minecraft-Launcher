use crate::models::{Account, AccountKind};
use crate::paths::accounts_path;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceLoginStart {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
    pub expires_in: u64,
}

/// Legacy Minecraft Launcher MSA client (device-code + MBI_SSL).
const MSA_CLIENT_ID: &str = "00000000402b5328";

pub fn load_accounts() -> Result<Vec<Account>, String> {
    let path = accounts_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_accounts(accounts: &[Account]) -> Result<(), String> {
    let path = accounts_path()?;
    let raw = serde_json::to_string_pretty(accounts).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn list_accounts() -> Result<Vec<Account>, String> {
    load_accounts()
}

pub fn begin_ms_login() -> Result<DeviceLoginStart, String> {
    let client = reqwest::blocking::Client::new();
    let body = format!(
        "client_id={}&scope={}&response_type=device_code",
        urlencoding::encode(MSA_CLIENT_ID),
        urlencoding::encode("service::user.auth.xboxlive.com::MBI_SSL"),
    );
    let res = client
        .post("https://login.live.com/oauth20_connect.srf")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct DeviceResp {
        user_code: String,
        device_code: String,
        verification_uri: String,
        expires_in: u64,
        interval: u64,
    }

    let body: DeviceResp = res.json().map_err(|e| e.to_string())?;
    Ok(DeviceLoginStart {
        user_code: body.user_code,
        verification_uri: body.verification_uri,
        device_code: body.device_code,
        interval: body.interval,
        expires_in: body.expires_in,
    })
}

pub fn poll_ms_login(device_code: String) -> Result<Option<Account>, String> {
    let client = reqwest::blocking::Client::new();
    let body = format!(
        "grant_type={}&client_id={}&device_code={}",
        urlencoding::encode("urn:ietf:params:oauth:grant-type:device_code"),
        urlencoding::encode(MSA_CLIENT_ID),
        urlencoding::encode(&device_code),
    );
    let res = client
        .post("https://login.live.com/oauth20_token.srf")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() == 400 {
        let v: serde_json::Value = res.json().unwrap_or_default();
        let err = v.get("error").and_then(|e| e.as_str()).unwrap_or("");
        if err == "authorization_pending" || err == "slow_down" {
            return Ok(None);
        }
        return Err(format!("MSA login error: {v}"));
    }

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: String,
        refresh_token: Option<String>,
    }

    let token: TokenResp = res
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let (mc_token, username, uuid) = exchange_for_minecraft(&token.access_token)?;

    let mut accounts = load_accounts()?;
    for a in accounts.iter_mut() {
        a.active = false;
    }
    let account = Account {
        id: Uuid::new_v4().to_string(),
        username,
        uuid,
        access_token: mc_token,
        refresh_token: token.refresh_token.unwrap_or_default(),
        active: true,
        kind: AccountKind::Microsoft,
    };
    accounts.push(account.clone());
    save_accounts(&accounts)?;
    Ok(Some(account))
}

fn xbox_authenticate(msa_token: &str, use_d_prefix: bool) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::new();
    let rps_ticket = if use_d_prefix {
        format!("d={msa_token}")
    } else {
        msa_token.to_string()
    };

    let xbox_body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": rps_ticket,
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT",
    });

    let res = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&xbox_body)
        .send()
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Xbox authenticate {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn exchange_for_minecraft(msa_token: &str) -> Result<(String, String, String), String> {
    let client = reqwest::blocking::Client::new();

    // Legacy MBI_SSL tokens are raw RPS tickets — `d=` causes 401.
    // Azure OAuth tokens need `d=`. Try without first, then with.
    let xbox = match xbox_authenticate(msa_token, false) {
        Ok(v) => v,
        Err(first) => match xbox_authenticate(msa_token, true) {
            Ok(v) => v,
            Err(second) => {
                return Err(format!(
                    "Xbox Live auth failed.\nWithout d=: {first}\nWith d=: {second}\n\
                     If XErr mentions family/child, add the account to a Microsoft Family or sign in once at minecraft.net."
                ));
            }
        },
    };

    let xbox_token = xbox
        .get("Token")
        .and_then(|t| t.as_str())
        .ok_or("Missing Xbox token")?
        .to_string();
    let uhs = xbox["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("Missing uhs")?
        .to_string();

    let xsts_body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbox_token],
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT",
    });

    let xsts_res = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&xsts_body)
        .send()
        .map_err(|e| e.to_string())?;
    let xsts_status = xsts_res.status();
    let xsts_text = xsts_res.text().unwrap_or_default();
    if !xsts_status.is_success() {
        return Err(format_xsts_error(xsts_status, &xsts_text));
    }
    let xsts: serde_json::Value = serde_json::from_str(&xsts_text).map_err(|e| e.to_string())?;

    let xsts_token = xsts
        .get("Token")
        .and_then(|t| t.as_str())
        .ok_or("Missing XSTS token")?
        .to_string();

    let mc: serde_json::Value = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={uhs};{xsts_token}")
        }))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let access = mc
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or("Missing Minecraft access token")?
        .to_string();

    let profile: serde_json::Value = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&access)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Minecraft profile failed (own the game?): {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    let username = profile
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Player")
        .to_string();
    let uuid = profile
        .get("id")
        .and_then(|n| n.as_str())
        .unwrap_or("00000000000000000000000000000000")
        .to_string();

    Ok((access, username, uuid))
}

fn format_xsts_error(status: reqwest::StatusCode, text: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(xerr) = v.get("XErr").and_then(|x| x.as_u64()) {
            let hint = match xerr {
                2148916233 => "No Xbox profile — sign in once at https://www.minecraft.net then retry.",
                2148916238 => "Child account — an adult must add this Microsoft account to a Family.",
                _ => "See Xbox error details.",
            };
            return format!("XSTS {status} XErr={xerr}: {hint}\n{text}");
        }
    }
    format!("XSTS {status}: {text}")
}

pub fn select_account(id: String) -> Result<Vec<Account>, String> {
    let mut accounts = load_accounts()?;
    let mut found = false;
    for a in accounts.iter_mut() {
        a.active = a.id == id;
        if a.active {
            found = true;
        }
    }
    if !found {
        return Err("Account not found".into());
    }
    save_accounts(&accounts)?;
    Ok(accounts)
}

pub fn remove_account(id: String) -> Result<Vec<Account>, String> {
    let mut accounts = load_accounts()?;
    accounts.retain(|a| a.id != id);
    let needs_active = !accounts.is_empty() && !accounts.iter().any(|a| a.active);
    if needs_active {
        if let Some(first) = accounts.first_mut() {
            first.active = true;
        }
    }
    save_accounts(&accounts)?;
    Ok(accounts)
}

pub fn active_account() -> Result<Option<Account>, String> {
    Ok(load_accounts()?.into_iter().find(|a| a.active))
}

pub fn add_offline_account(username: String) -> Result<Vec<Account>, String> {
    let name = username.trim();
    if name.is_empty() {
        return Err("用户名不能为空".into());
    }
    let mut hasher = Sha1::new();
    hasher.update(format!("OfflinePlayer:{name}").as_bytes());
    let digest = hasher.finalize();
    let uuid = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        digest[0],
        digest[1],
        digest[2],
        digest[3],
        digest[4],
        digest[5],
        digest[6],
        digest[7],
        digest[8],
        digest[9],
        digest[10],
        digest[11],
        digest[12],
        digest[13],
        digest[14],
        digest[15]
    );

    let mut accounts = load_accounts()?;
    for a in accounts.iter_mut() {
        a.active = false;
    }
    accounts.push(Account {
        id: Uuid::new_v4().to_string(),
        username: name.to_string(),
        uuid,
        access_token: "0".into(),
        refresh_token: String::new(),
        active: true,
        kind: AccountKind::Offline,
    });
    save_accounts(&accounts)?;
    Ok(accounts)
}

/// Classic Yggdrasil login against LittleSkin (email + password).
pub fn add_littleskin_account(email: String, password: String) -> Result<Vec<Account>, String> {
    let email = email.trim().to_string();
    if email.is_empty() || password.is_empty() {
        return Err("LittleSkin email and password are required".into());
    }
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "agent": { "name": "Minecraft", "version": 1 },
        "username": email,
        "password": password,
        "requestUser": true,
    });
    let res = client
        .post("https://littleskin.cn/api/yggdrasil/authserver/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("User-Agent", "Northstar/1.1.1")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().unwrap_or_default();
    if !status.is_success() {
        return Err(format!("LittleSkin login failed ({status}): {text}"));
    }
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let access = data
        .get("accessToken")
        .and_then(|t| t.as_str())
        .ok_or("LittleSkin: missing accessToken")?
        .to_string();
    let profile = data
        .get("selectedProfile")
        .or_else(|| {
            data.get("availableProfiles")
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
        })
        .ok_or("LittleSkin: no player profile on this account")?;
    let username = profile
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Player")
        .to_string();
    let uuid = profile
        .get("id")
        .and_then(|n| n.as_str())
        .unwrap_or("00000000000000000000000000000000")
        .to_string();

    let mut accounts = load_accounts()?;
    for a in accounts.iter_mut() {
        a.active = false;
    }
    accounts.push(Account {
        id: Uuid::new_v4().to_string(),
        username,
        uuid,
        access_token: access,
        refresh_token: email,
        active: true,
        kind: AccountKind::LittleSkin,
    });
    save_accounts(&accounts)?;
    Ok(accounts)
}
