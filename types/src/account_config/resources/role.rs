// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::resources::{
    ChildVASP, Credential, DesignatedDealer, DesignatedDealerPreburns, DiemIdDomainManager,
    DiemIdDomains, ParentVASP,
};
use serde::{Deserialize, Serialize};

/// A enum that captures the collection of role-specific resources stored under each account type
#[derive(Debug, Serialize, Deserialize)]
pub enum AccountRole {
    ParentVASP {
        vasp: ParentVASP,
        credential: Credential,
        diem_id_domains: Option<DiemIdDomains>,
    },
    ChildVASP(ChildVASP),
    DesignatedDealer {
        dd_credential: Credential,
        preburn_balances: DesignatedDealerPreburns,
        designated_dealer: DesignatedDealer,
    },
    TreasuryCompliance {
        diem_id_domain_manager: Option<DiemIdDomainManager>,
    },
    Unknown,
    // TODO: add other roles
}
