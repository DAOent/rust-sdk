use crate::model::dao::WithGov;
use crate::{account, Client};

use super::base_hander::BaseHander;
use super::wetee_gov::run_sudo_or_gov;
use sp_core::{crypto::Ss58Codec, sr25519::Public};
use wetee_project::ProjectInfo;
use wetee_runtime::{AccountId, Runtime, RuntimeCall, Signature, WeteeProjectCall};

use substrate_api_client::{ExtrinsicSigner, GetStorage, SubmitAndWatchUntilSuccess};

/// 账户
pub struct WeteeProject {
    pub base: BaseHander,
}

impl WeteeProject {
    pub fn new(c: Client) -> Self {
        Self {
            base: BaseHander::new(c, false),
        }
    }

    pub fn project_list(
        &mut self,
        dao_id: u64,
    ) -> anyhow::Result<Vec<ProjectInfo<AccountId>>, anyhow::Error> {
        let api = self.base.get_client()?;

        // 构建请求
        let result: Vec<ProjectInfo<AccountId>> = api
            .get_storage_map("WeteeProject", "DaoProjects", dao_id, None)
            .unwrap()
            .unwrap_or_else(|| vec![]);

        Ok(result)
    }

    // 创建项目
    pub fn create_project(
        &mut self,
        from: String,
        dao_id: u64,
        name: String,
        desc: String,
        ext: Option<WithGov>,
    ) -> anyhow::Result<(), anyhow::Error> {
        let mut api = self.base.get_client()?;

        let from_pair = account::get_from_address(from.clone())?;
        api.set_signer(ExtrinsicSigner::<_, Signature, Runtime>::new(from_pair));

        // 构建请求
        let call = RuntimeCall::WeteeProject(WeteeProjectCall::create_project {
            name: name.into(),
            description: desc.into(),
            dao_id,
            creator: AccountId::from(Public::from_string(&from).unwrap()),
        });

        if ext.is_some() {
            return run_sudo_or_gov(api, dao_id, call, ext.unwrap());
        }

        let signer_nonce = api.get_nonce().unwrap();
        let xt = api.compose_extrinsic_offline(call, signer_nonce);

        // 发送请求
        let result = api.submit_and_watch_extrinsic_until_success(xt, false);

        match result {
            Ok(report) => {
                println!(
                    "[+] Extrinsic got included in block {:?}",
                    report.block_hash
                );
                return Ok(());
            }
            Err(e) => {
                println!("[+] Couldn't execute the extrinsic due to {:?}\n", e);
                let string_error = format!("{:?}", e);
                return Err(anyhow::anyhow!(string_error));
            }
        };
    }
}
