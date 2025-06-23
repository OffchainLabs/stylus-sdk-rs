// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

impl AuthArgs {
    fn build_wallet(&self, chain_id: u64) -> eyre::Result<EthereumWallet> {
        if let Some(key) = &self.private_key {
            if key.is_empty() {
                return Err(eyre!("empty private key"));
            }
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        if let Some(file) = &self.private_key_path {
            let key = fs::read_to_string(file).wrap_err("could not open private key file")?;
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        let keystore = self.keystore_path.as_ref().ok_or(eyre!("no keystore"))?;
        let password = self
            .keystore_password_path
            .as_ref()
            .map(fs::read_to_string)
            .unwrap_or(Ok("".into()))?;

        let signer =
            LocalSigner::decrypt_keystore(keystore, password)?.with_chain_id(Some(chain_id));
        Ok(EthereumWallet::new(signer))
    }
}

impl ProviderArgs {
    pub async fn build_provider(&self) -> eyre::Result<impl Provider> {
        let provider = ProviderBuilder::new().connect(&self.endpoint).await?;
        Ok(provider)
    }

    pub async fn build_provider_with_wallet(
        &self,
        auth: &AuthArgs,
    ) -> eyre::Result<impl Provider + WalletProvider> {
        let provider = self.build_provider().await?;
        let chain_id = provider.get_chain_id().await?;
        let wallet = auth.build_wallet(chain_id)?;
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(&self.endpoint)
            .await?;
        Ok(provider)
    }
}
