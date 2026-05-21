/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/agc_solana.json`.
 */
export type AgcSolana = {
  "address": "H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC",
  "metadata": {
    "name": "agcSolana",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Solana implementation of Agent Credit Protocol core policy and vault mechanics"
  },
  "instructions": [
    {
      "name": "acceptAdmin",
      "discriminator": [
        112,
        42,
        45,
        90,
        116,
        181,
        13,
        170
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "pendingAdmin",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "burnTreasuryAgc",
      "discriminator": [
        181,
        74,
        231,
        5,
        46,
        199,
        31,
        200
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "treasuryAgc",
          "writable": true
        },
        {
          "name": "treasuryAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelBuybackCampaign",
      "discriminator": [
        40,
        169,
        154,
        102,
        45,
        47,
        38,
        193
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "campaign",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "campaign.campaign_id",
                "account": "buybackCampaign"
              }
            ]
          }
        },
        {
          "name": "campaignUsdcEscrow",
          "writable": true
        },
        {
          "name": "campaignAgcVault",
          "writable": true
        },
        {
          "name": "treasuryUsdc",
          "writable": true
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "campaignAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "campaign.campaign_id",
                "account": "buybackCampaign"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "depositCreditCollateral",
      "discriminator": [
        56,
        40,
        41,
        171,
        236,
        222,
        91,
        69
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "credit_line.borrower",
                "account": "creditLine"
              },
              {
                "kind": "account",
                "path": "credit_line.line_id",
                "account": "creditLine"
              }
            ]
          }
        },
        {
          "name": "borrower",
          "signer": true
        },
        {
          "name": "borrowerCollateral",
          "writable": true
        },
        {
          "name": "collateralVault",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "depositUnderwriterAgc",
      "discriminator": [
        47,
        254,
        189,
        172,
        1,
        29,
        131,
        2
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "underwriter",
          "writable": true,
          "signer": true
        },
        {
          "name": "underwriterAgc",
          "writable": true
        },
        {
          "name": "underwriterVaultAgc",
          "writable": true
        },
        {
          "name": "underwriterPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  110,
                  100,
                  101,
                  114,
                  119,
                  114,
                  105,
                  116,
                  101,
                  114,
                  45,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "underwriter"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "depositXagc",
      "discriminator": [
        108,
        201,
        241,
        185,
        120,
        223,
        188,
        147
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "depositor",
          "signer": true
        },
        {
          "name": "depositorAgc",
          "writable": true
        },
        {
          "name": "xagcVaultAgc",
          "writable": true
        },
        {
          "name": "xagcMint",
          "writable": true
        },
        {
          "name": "receiverXagc",
          "writable": true
        },
        {
          "name": "mintAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "assets",
          "type": "u64"
        }
      ]
    },
    {
      "name": "drawCreditLine",
      "discriminator": [
        244,
        240,
        156,
        222,
        214,
        114,
        210,
        43
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "collateralAsset"
        },
        {
          "name": "collateralOracle",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  111,
                  114,
                  97,
                  99,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility.collateral_mint",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "creditLine",
          "writable": true
        },
        {
          "name": "borrower",
          "signer": true
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "borrowerAgcDestination",
          "writable": true
        },
        {
          "name": "treasuryAgc",
          "writable": true
        },
        {
          "name": "mintAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "executeBuybackTwapSlice",
      "discriminator": [
        33,
        11,
        76,
        140,
        49,
        108,
        21,
        113
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "campaign",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "campaign.campaign_id",
                "account": "buybackCampaign"
              }
            ]
          }
        },
        {
          "name": "campaignUsdcEscrow",
          "writable": true
        },
        {
          "name": "campaignAgcVault",
          "writable": true
        },
        {
          "name": "adapterUsdcDestination",
          "writable": true
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "campaignAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "campaign.campaign_id",
                "account": "buybackCampaign"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "buybackSliceArgs"
            }
          }
        }
      ]
    },
    {
      "name": "initializeCreditFacility",
      "discriminator": [
        134,
        28,
        164,
        57,
        219,
        112,
        80,
        35
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "collateralMint"
        },
        {
          "name": "collateralAsset",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "collateralMint"
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "arg",
                "path": "facilityId"
              }
            ]
          }
        },
        {
          "name": "facilityAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "collateralVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "underwriterVaultAgc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  110,
                  100,
                  101,
                  114,
                  119,
                  114,
                  105,
                  116,
                  101,
                  114,
                  45,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "facilityId",
          "type": "u64"
        },
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "creditFacilityConfig"
            }
          }
        }
      ]
    },
    {
      "name": "initializeProtocol",
      "discriminator": [
        188,
        233,
        252,
        106,
        134,
        146,
        202,
        91
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "admin",
          "signer": true
        },
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "xagcMint",
          "writable": true
        },
        {
          "name": "usdcMint"
        },
        {
          "name": "treasuryAgc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  97,
                  103,
                  99
                ]
              }
            ]
          }
        },
        {
          "name": "treasuryUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  117,
                  115,
                  100,
                  99
                ]
              }
            ]
          }
        },
        {
          "name": "xagcVaultAgc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  120,
                  97,
                  103,
                  99,
                  45,
                  118,
                  97,
                  117,
                  108,
                  116,
                  45,
                  97,
                  103,
                  99
                ]
              }
            ]
          }
        },
        {
          "name": "mintAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "treasuryAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "xagcAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  120,
                  97,
                  103,
                  99,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "initializeProtocolArgs"
            }
          }
        }
      ]
    },
    {
      "name": "markCreditLineDefault",
      "discriminator": [
        217,
        239,
        63,
        153,
        180,
        253,
        177,
        227
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "collateralAsset"
        },
        {
          "name": "collateralOracle",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  111,
                  114,
                  97,
                  99,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility.collateral_mint",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "credit_line.borrower",
                "account": "creditLine"
              },
              {
                "kind": "account",
                "path": "credit_line.line_id",
                "account": "creditLine"
              }
            ]
          }
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "underwriterVaultAgc",
          "writable": true
        },
        {
          "name": "facilityAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "openCreditLine",
      "discriminator": [
        126,
        111,
        245,
        44,
        233,
        249,
        30,
        238
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "borrower"
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "borrower"
              },
              {
                "kind": "arg",
                "path": "lineId"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "lineId",
          "type": "u64"
        },
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "openCreditLineArgs"
            }
          }
        }
      ]
    },
    {
      "name": "recordMarketObservation",
      "discriminator": [
        27,
        235,
        76,
        215,
        122,
        161,
        52,
        147
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        }
      ],
      "args": [
        {
          "name": "priceX18",
          "type": "u128"
        }
      ]
    },
    {
      "name": "recordSwap",
      "discriminator": [
        164,
        158,
        148,
        54,
        167,
        137,
        171,
        59
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "recordSwapArgs"
            }
          }
        }
      ]
    },
    {
      "name": "redeemXagc",
      "discriminator": [
        150,
        113,
        149,
        152,
        41,
        99,
        43,
        76
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "ownerXagc",
          "writable": true
        },
        {
          "name": "xagcMint",
          "writable": true
        },
        {
          "name": "xagcVaultAgc",
          "writable": true
        },
        {
          "name": "treasuryAgc",
          "writable": true
        },
        {
          "name": "receiverAgc",
          "writable": true
        },
        {
          "name": "xagcAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  120,
                  97,
                  103,
                  99,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "shares",
          "type": "u64"
        }
      ]
    },
    {
      "name": "refreshCollateralOracleFromPyth",
      "discriminator": [
        211,
        32,
        144,
        150,
        173,
        128,
        165,
        45
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "mint"
        },
        {
          "name": "collateralAsset",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "collateralOracle",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  111,
                  114,
                  97,
                  99,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "priceUpdate"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "repayCreditLine",
      "discriminator": [
        105,
        166,
        167,
        5,
        56,
        41,
        160,
        171
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "credit_line.borrower",
                "account": "creditLine"
              },
              {
                "kind": "account",
                "path": "credit_line.line_id",
                "account": "creditLine"
              }
            ]
          }
        },
        {
          "name": "payer",
          "signer": true
        },
        {
          "name": "payerAgc",
          "writable": true
        },
        {
          "name": "underwriterVaultAgc",
          "writable": true
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "reserveTreasuryBuybackUsdc",
      "discriminator": [
        113,
        3,
        255,
        229,
        153,
        91,
        179,
        112
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "treasuryUsdc",
          "writable": true
        },
        {
          "name": "buybackUsdcDestination",
          "writable": true
        },
        {
          "name": "treasuryAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "seizeDefaultedCollateral",
      "discriminator": [
        157,
        136,
        21,
        68,
        118,
        54,
        242,
        19
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "collateralAsset"
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "credit_line.borrower",
                "account": "creditLine"
              },
              {
                "kind": "account",
                "path": "credit_line.line_id",
                "account": "creditLine"
              }
            ]
          }
        },
        {
          "name": "collateralVault",
          "writable": true
        },
        {
          "name": "collateralDestination",
          "writable": true
        },
        {
          "name": "facilityAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "setBuybackUsdcEscrow",
      "discriminator": [
        201,
        174,
        186,
        209,
        181,
        176,
        243,
        215
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "state"
          ]
        },
        {
          "name": "buybackUsdcEscrow"
        }
      ],
      "args": []
    },
    {
      "name": "setCollateralAsset",
      "discriminator": [
        63,
        82,
        188,
        17,
        222,
        188,
        109,
        40
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "mint"
        },
        {
          "name": "collateralAsset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "collateralAssetConfig"
            }
          }
        }
      ]
    },
    {
      "name": "setCollateralOraclePrice",
      "discriminator": [
        32,
        122,
        28,
        139,
        77,
        250,
        140,
        240
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "mint"
        },
        {
          "name": "collateralAsset",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "collateralOracle",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  111,
                  114,
                  97,
                  99,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "price",
          "type": {
            "defined": {
              "name": "collateralOraclePriceInput"
            }
          }
        }
      ]
    },
    {
      "name": "setCreditFacilityConfig",
      "discriminator": [
        50,
        131,
        43,
        251,
        24,
        127,
        46,
        79
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "collateralAsset"
        }
      ],
      "args": [
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "creditFacilityConfig"
            }
          }
        }
      ]
    },
    {
      "name": "setExitFeeBps",
      "discriminator": [
        116,
        221,
        63,
        69,
        230,
        122,
        216,
        161
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "exitFeeBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setGovernanceAuthorities",
      "discriminator": [
        176,
        224,
        97,
        63,
        100,
        187,
        20,
        34
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "state"
          ]
        }
      ],
      "args": [
        {
          "name": "authorities",
          "type": {
            "defined": {
              "name": "governanceAuthorities"
            }
          }
        }
      ]
    },
    {
      "name": "setGrowthProgramsEnabled",
      "discriminator": [
        163,
        88,
        159,
        16,
        153,
        3,
        50,
        39
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "enabled",
          "type": "bool"
        }
      ]
    },
    {
      "name": "setKeeper",
      "discriminator": [
        102,
        94,
        23,
        78,
        157,
        222,
        243,
        214
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true,
          "relations": [
            "state"
          ]
        },
        {
          "name": "keeperAuthority"
        },
        {
          "name": "keeper",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  101,
                  112,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "keeperAuthority"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "allowed",
          "type": "bool"
        }
      ]
    },
    {
      "name": "setKeeperPermissions",
      "discriminator": [
        49,
        51,
        150,
        111,
        8,
        36,
        14,
        22
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true,
          "relations": [
            "state"
          ]
        },
        {
          "name": "keeperAuthority"
        },
        {
          "name": "keeper",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  101,
                  112,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "keeperAuthority"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "permissions",
          "type": {
            "defined": {
              "name": "keeperPermissions"
            }
          }
        }
      ]
    },
    {
      "name": "setMarketAdapterAuthority",
      "discriminator": [
        241,
        17,
        231,
        137,
        153,
        77,
        153,
        147
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "state"
          ]
        }
      ],
      "args": [
        {
          "name": "authority",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "setMintDistribution",
      "discriminator": [
        185,
        81,
        50,
        132,
        155,
        17,
        185,
        125
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "distribution",
          "type": {
            "defined": {
              "name": "mintDistribution"
            }
          }
        }
      ]
    },
    {
      "name": "setPauseFlags",
      "discriminator": [
        205,
        167,
        85,
        237,
        144,
        202,
        248,
        175
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "pauseFlags",
          "type": {
            "defined": {
              "name": "pauseFlags"
            }
          }
        }
      ]
    },
    {
      "name": "setPolicyParams",
      "discriminator": [
        170,
        27,
        63,
        174,
        155,
        242,
        215,
        118
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "policyParams"
            }
          }
        }
      ]
    },
    {
      "name": "setPythReceiverProgram",
      "discriminator": [
        141,
        221,
        223,
        120,
        194,
        152,
        205,
        191
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "state"
          ]
        }
      ],
      "args": [
        {
          "name": "receiverProgram",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "setSettlementRecipients",
      "discriminator": [
        160,
        89,
        18,
        180,
        138,
        235,
        150,
        240
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "recipients",
          "type": {
            "defined": {
              "name": "settlementRecipients"
            }
          }
        }
      ]
    },
    {
      "name": "settleEpoch",
      "discriminator": [
        148,
        223,
        178,
        38,
        201,
        158,
        167,
        13
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "agcMint",
          "writable": true
        },
        {
          "name": "xagcMint"
        },
        {
          "name": "xagcVaultAgc",
          "writable": true
        },
        {
          "name": "treasuryAgc",
          "writable": true
        },
        {
          "name": "treasuryUsdc"
        },
        {
          "name": "growthProgramsAgc",
          "writable": true
        },
        {
          "name": "lpAgc",
          "writable": true
        },
        {
          "name": "integratorsAgc",
          "writable": true
        },
        {
          "name": "mintAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "externalMetrics",
          "type": {
            "defined": {
              "name": "externalMetrics"
            }
          }
        }
      ]
    },
    {
      "name": "startBuybackCampaign",
      "discriminator": [
        53,
        145,
        4,
        225,
        250,
        163,
        189,
        143
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "keeper"
        },
        {
          "name": "treasuryUsdc",
          "writable": true
        },
        {
          "name": "campaign",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110
                ]
              },
              {
                "kind": "arg",
                "path": "campaignId"
              }
            ]
          }
        },
        {
          "name": "campaignAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "arg",
                "path": "campaignId"
              }
            ]
          }
        },
        {
          "name": "campaignUsdcEscrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110,
                  45,
                  117,
                  115,
                  100,
                  99
                ]
              },
              {
                "kind": "arg",
                "path": "campaignId"
              }
            ]
          }
        },
        {
          "name": "campaignAgcVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  117,
                  121,
                  98,
                  97,
                  99,
                  107,
                  45,
                  99,
                  97,
                  109,
                  112,
                  97,
                  105,
                  103,
                  110,
                  45,
                  97,
                  103,
                  99
                ]
              },
              {
                "kind": "arg",
                "path": "campaignId"
              }
            ]
          }
        },
        {
          "name": "usdcMint"
        },
        {
          "name": "agcMint"
        },
        {
          "name": "treasuryAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "campaignId",
          "type": "u64"
        },
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "buybackCampaignConfig"
            }
          }
        }
      ]
    },
    {
      "name": "transferAdmin",
      "discriminator": [
        42,
        242,
        66,
        106,
        228,
        10,
        111,
        156
      ],
      "accounts": [
        {
          "name": "state",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "state"
          ]
        }
      ],
      "args": [
        {
          "name": "nextAdmin",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "withdrawCreditCollateral",
      "discriminator": [
        31,
        57,
        83,
        135,
        246,
        178,
        252,
        254
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "collateralAsset"
        },
        {
          "name": "collateralOracle",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  108,
                  108,
                  97,
                  116,
                  101,
                  114,
                  97,
                  108,
                  45,
                  111,
                  114,
                  97,
                  99,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility.collateral_mint",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "creditLine",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  108,
                  105,
                  110,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "credit_line.borrower",
                "account": "creditLine"
              },
              {
                "kind": "account",
                "path": "credit_line.line_id",
                "account": "creditLine"
              }
            ]
          }
        },
        {
          "name": "borrower",
          "signer": true
        },
        {
          "name": "collateralVault",
          "writable": true
        },
        {
          "name": "borrowerCollateralDestination",
          "writable": true
        },
        {
          "name": "facilityAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdrawUnderwriterAgc",
      "discriminator": [
        105,
        241,
        235,
        158,
        191,
        165,
        246,
        175
      ],
      "accounts": [
        {
          "name": "state",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "facility",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility.facility_id",
                "account": "creditFacility"
              }
            ]
          }
        },
        {
          "name": "underwriter",
          "signer": true
        },
        {
          "name": "underwriterPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  110,
                  100,
                  101,
                  114,
                  119,
                  114,
                  105,
                  116,
                  101,
                  114,
                  45,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              },
              {
                "kind": "account",
                "path": "underwriter"
              }
            ]
          }
        },
        {
          "name": "underwriterVaultAgc",
          "writable": true
        },
        {
          "name": "underwriterAgcDestination",
          "writable": true
        },
        {
          "name": "facilityAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  45,
                  102,
                  97,
                  99,
                  105,
                  108,
                  105,
                  116,
                  121,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "facility"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "shares",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "buybackCampaign",
      "discriminator": [
        73,
        198,
        70,
        190,
        124,
        31,
        13,
        5
      ]
    },
    {
      "name": "collateralAsset",
      "discriminator": [
        254,
        180,
        112,
        90,
        72,
        6,
        245,
        149
      ]
    },
    {
      "name": "collateralOracle",
      "discriminator": [
        7,
        36,
        166,
        11,
        132,
        196,
        0,
        245
      ]
    },
    {
      "name": "creditFacility",
      "discriminator": [
        192,
        187,
        81,
        192,
        233,
        124,
        22,
        17
      ]
    },
    {
      "name": "creditLine",
      "discriminator": [
        220,
        226,
        205,
        24,
        220,
        151,
        129,
        104
      ]
    },
    {
      "name": "keeper",
      "discriminator": [
        127,
        221,
        194,
        46,
        120,
        73,
        144,
        77
      ]
    },
    {
      "name": "protocolState",
      "discriminator": [
        33,
        51,
        173,
        134,
        35,
        140,
        195,
        248
      ]
    },
    {
      "name": "underwriterPosition",
      "discriminator": [
        33,
        87,
        62,
        126,
        208,
        123,
        202,
        22
      ]
    }
  ],
  "events": [
    {
      "name": "adminTransferStarted",
      "discriminator": [
        31,
        82,
        117,
        177,
        147,
        168,
        210,
        177
      ]
    },
    {
      "name": "adminTransferred",
      "discriminator": [
        255,
        147,
        182,
        5,
        199,
        217,
        38,
        179
      ]
    },
    {
      "name": "buybackCampaignCancelled",
      "discriminator": [
        170,
        143,
        194,
        187,
        47,
        172,
        178,
        36
      ]
    },
    {
      "name": "buybackCampaignStarted",
      "discriminator": [
        235,
        255,
        85,
        235,
        228,
        56,
        141,
        241
      ]
    },
    {
      "name": "buybackTwapSliceExecuted",
      "discriminator": [
        69,
        251,
        12,
        103,
        219,
        149,
        60,
        185
      ]
    },
    {
      "name": "buybackUsdcEscrowUpdated",
      "discriminator": [
        189,
        206,
        22,
        81,
        69,
        222,
        126,
        66
      ]
    },
    {
      "name": "collateralAssetUpdated",
      "discriminator": [
        109,
        182,
        169,
        21,
        170,
        245,
        253,
        90
      ]
    },
    {
      "name": "collateralOraclePriceUpdated",
      "discriminator": [
        39,
        10,
        128,
        204,
        138,
        140,
        44,
        114
      ]
    },
    {
      "name": "creditCollateralDeposited",
      "discriminator": [
        12,
        124,
        149,
        176,
        51,
        86,
        90,
        54
      ]
    },
    {
      "name": "creditCollateralWithdrawn",
      "discriminator": [
        236,
        193,
        138,
        186,
        27,
        68,
        102,
        229
      ]
    },
    {
      "name": "creditFacilityConfigUpdated",
      "discriminator": [
        118,
        103,
        65,
        236,
        152,
        172,
        78,
        19
      ]
    },
    {
      "name": "creditFacilityInitialized",
      "discriminator": [
        35,
        108,
        129,
        242,
        44,
        159,
        197,
        186
      ]
    },
    {
      "name": "creditLineDefaulted",
      "discriminator": [
        51,
        30,
        136,
        186,
        118,
        147,
        147,
        196
      ]
    },
    {
      "name": "creditLineDrawn",
      "discriminator": [
        128,
        226,
        93,
        77,
        149,
        53,
        111,
        84
      ]
    },
    {
      "name": "creditLineOpened",
      "discriminator": [
        72,
        103,
        68,
        151,
        111,
        168,
        27,
        89
      ]
    },
    {
      "name": "creditLineRepaid",
      "discriminator": [
        201,
        66,
        159,
        5,
        169,
        142,
        192,
        187
      ]
    },
    {
      "name": "defaultedCollateralSeized",
      "discriminator": [
        129,
        115,
        255,
        172,
        208,
        252,
        174,
        8
      ]
    },
    {
      "name": "epochSettled",
      "discriminator": [
        32,
        219,
        45,
        156,
        250,
        115,
        190,
        255
      ]
    },
    {
      "name": "exitFeeUpdated",
      "discriminator": [
        36,
        119,
        54,
        25,
        253,
        35,
        51,
        232
      ]
    },
    {
      "name": "governanceAuthoritiesUpdated",
      "discriminator": [
        149,
        244,
        221,
        178,
        44,
        4,
        8,
        161
      ]
    },
    {
      "name": "growthProgramsEnabledUpdated",
      "discriminator": [
        226,
        184,
        5,
        91,
        27,
        107,
        11,
        21
      ]
    },
    {
      "name": "keeperPermissionsUpdated",
      "discriminator": [
        9,
        237,
        2,
        236,
        132,
        245,
        185,
        128
      ]
    },
    {
      "name": "marketAdapterAuthorityUpdated",
      "discriminator": [
        59,
        43,
        53,
        103,
        253,
        246,
        79,
        184
      ]
    },
    {
      "name": "mintDistributionUpdated",
      "discriminator": [
        139,
        190,
        207,
        127,
        235,
        68,
        75,
        235
      ]
    },
    {
      "name": "pauseFlagsUpdated",
      "discriminator": [
        27,
        241,
        6,
        218,
        123,
        5,
        234,
        94
      ]
    },
    {
      "name": "policyParametersUpdated",
      "discriminator": [
        84,
        165,
        179,
        98,
        151,
        206,
        123,
        173
      ]
    },
    {
      "name": "protocolInitialized",
      "discriminator": [
        173,
        122,
        168,
        254,
        9,
        118,
        76,
        132
      ]
    },
    {
      "name": "pythReceiverProgramUpdated",
      "discriminator": [
        36,
        29,
        95,
        66,
        72,
        65,
        173,
        41
      ]
    },
    {
      "name": "settlementRecipientsUpdated",
      "discriminator": [
        104,
        77,
        134,
        53,
        173,
        169,
        80,
        109
      ]
    },
    {
      "name": "swapRecorded",
      "discriminator": [
        59,
        50,
        246,
        150,
        31,
        55,
        2,
        219
      ]
    },
    {
      "name": "treasuryAgcBurned",
      "discriminator": [
        73,
        228,
        159,
        107,
        230,
        0,
        250,
        95
      ]
    },
    {
      "name": "treasuryBuybackUsdcReserved",
      "discriminator": [
        31,
        167,
        185,
        74,
        230,
        157,
        185,
        85
      ]
    },
    {
      "name": "underwriterAgcDeposited",
      "discriminator": [
        223,
        119,
        147,
        177,
        82,
        158,
        188,
        166
      ]
    },
    {
      "name": "underwriterAgcWithdrawn",
      "discriminator": [
        190,
        29,
        127,
        239,
        63,
        148,
        225,
        155
      ]
    },
    {
      "name": "xagcDeposited",
      "discriminator": [
        171,
        70,
        208,
        161,
        37,
        174,
        30,
        184
      ]
    },
    {
      "name": "xagcRedeemed",
      "discriminator": [
        4,
        164,
        218,
        250,
        23,
        166,
        92,
        124
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "unauthorized",
      "msg": "The signer is not authorized for this instruction."
    },
    {
      "code": 6001,
      "name": "invalidMintAuthority",
      "msg": "The configured SPL token mint authority is not the AGC program PDA."
    },
    {
      "code": 6002,
      "name": "invalidFreezeAuthority",
      "msg": "The AGC-controlled mint must not have an external freeze authority."
    },
    {
      "code": 6003,
      "name": "invalidTokenAccount",
      "msg": "The provided token account does not match the protocol configuration."
    },
    {
      "code": 6004,
      "name": "unsupportedDecimalConfig",
      "msg": "The mint decimals are unsupported."
    },
    {
      "code": 6005,
      "name": "invalidMintDistribution",
      "msg": "The mint distribution must total 10_000 bps and include xAGC."
    },
    {
      "code": 6006,
      "name": "invalidFee",
      "msg": "The fee must be below 10_000 bps."
    },
    {
      "code": 6007,
      "name": "invalidPolicyParams",
      "msg": "The policy parameters are internally inconsistent."
    },
    {
      "code": 6008,
      "name": "zeroAmount",
      "msg": "The amount must be non-zero."
    },
    {
      "code": 6009,
      "name": "insufficientShares",
      "msg": "The xAGC token account does not have enough shares."
    },
    {
      "code": 6010,
      "name": "invalidPrice",
      "msg": "The market price must be non-zero."
    },
    {
      "code": 6011,
      "name": "epochTooSoon",
      "msg": "The epoch cannot be settled yet."
    },
    {
      "code": 6012,
      "name": "invalidEpoch",
      "msg": "The epoch id has already been settled."
    },
    {
      "code": 6013,
      "name": "invalidSettlementRecipient",
      "msg": "A settlement recipient account does not match protocol state."
    },
    {
      "code": 6014,
      "name": "noPendingTreasuryBuyback",
      "msg": "There is no queued treasury buyback budget."
    },
    {
      "code": 6015,
      "name": "buybackEscrowNotConfigured",
      "msg": "The protocol buyback escrow has not been configured."
    },
    {
      "code": 6016,
      "name": "invalidBuybackEscrow",
      "msg": "The provided buyback escrow is not the configured escrow."
    },
    {
      "code": 6017,
      "name": "paused",
      "msg": "The protocol is paused for this instruction."
    },
    {
      "code": 6018,
      "name": "invalidAdmin",
      "msg": "The requested admin is invalid."
    },
    {
      "code": 6019,
      "name": "invalidGovernanceAuthority",
      "msg": "The requested governance authority is invalid."
    },
    {
      "code": 6020,
      "name": "invalidCollateralAssetConfig",
      "msg": "The collateral asset configuration is invalid."
    },
    {
      "code": 6021,
      "name": "invalidOracleSource",
      "msg": "The collateral oracle source is invalid for this instruction."
    },
    {
      "code": 6022,
      "name": "invalidOraclePrice",
      "msg": "The cached oracle price is invalid or stale."
    },
    {
      "code": 6023,
      "name": "collateralDisabled",
      "msg": "The collateral asset is disabled."
    },
    {
      "code": 6024,
      "name": "invalidCreditFacilityConfig",
      "msg": "The credit facility configuration is invalid."
    },
    {
      "code": 6025,
      "name": "invalidCreditLineConfig",
      "msg": "The credit line configuration is invalid."
    },
    {
      "code": 6026,
      "name": "creditFacilityInactive",
      "msg": "The credit facility is not active."
    },
    {
      "code": 6027,
      "name": "creditLineInactive",
      "msg": "The credit line is not active."
    },
    {
      "code": 6028,
      "name": "creditLimitExceeded",
      "msg": "The requested draw exceeds the credit line limit."
    },
    {
      "code": 6029,
      "name": "insufficientCollateral",
      "msg": "The credit line does not have enough collateral."
    },
    {
      "code": 6030,
      "name": "insufficientCreditHealth",
      "msg": "The credit line would be undercollateralized."
    },
    {
      "code": 6031,
      "name": "insufficientUnderwriterReserve",
      "msg": "The underwriter vault would fall below required reserves."
    },
    {
      "code": 6032,
      "name": "creditLineMatured",
      "msg": "The credit line has already matured."
    },
    {
      "code": 6033,
      "name": "noOutstandingDebt",
      "msg": "The credit line has no outstanding debt."
    },
    {
      "code": 6034,
      "name": "creditLineHealthy",
      "msg": "The credit line is still healthy."
    },
    {
      "code": 6035,
      "name": "creditLineNotDefaulted",
      "msg": "The credit line is not defaulted."
    },
    {
      "code": 6036,
      "name": "deprecatedBuybackPath",
      "msg": "The legacy buyback transfer path is disabled; use buyback campaigns."
    },
    {
      "code": 6037,
      "name": "invalidBuybackCampaignConfig",
      "msg": "The buyback campaign configuration is invalid."
    },
    {
      "code": 6038,
      "name": "buybackCampaignInactive",
      "msg": "The buyback campaign is not active."
    },
    {
      "code": 6039,
      "name": "buybackCampaignNotReady",
      "msg": "The buyback campaign or slice is not ready."
    },
    {
      "code": 6040,
      "name": "buybackSliceTooLarge",
      "msg": "The buyback slice exceeds campaign limits."
    },
    {
      "code": 6041,
      "name": "buybackDeadlineExpired",
      "msg": "The buyback slice deadline has expired."
    },
    {
      "code": 6042,
      "name": "insufficientBuybackOutput",
      "msg": "The buyback slice did not deliver enough AGC to burn."
    },
    {
      "code": 6043,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow or underflow."
    },
    {
      "code": 6044,
      "name": "amountTooLarge",
      "msg": "A u128 policy amount does not fit into a u64 SPL token amount."
    },
    {
      "code": 6045,
      "name": "invalidClock",
      "msg": "Clock returned a negative timestamp."
    }
  ],
  "types": [
    {
      "name": "adminTransferStarted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "currentAdmin",
            "type": "pubkey"
          },
          {
            "name": "pendingAdmin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "adminTransferred",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "previousAdmin",
            "type": "pubkey"
          },
          {
            "name": "newAdmin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "assetClass",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "stable"
          },
          {
            "name": "btc"
          },
          {
            "name": "rwa"
          },
          {
            "name": "other"
          }
        ]
      }
    },
    {
      "name": "buybackCampaign",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "campaignId",
            "type": "u64"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "buybackCampaignStatus"
              }
            }
          },
          {
            "name": "totalUsdc",
            "type": "u64"
          },
          {
            "name": "remainingUsdc",
            "type": "u64"
          },
          {
            "name": "spentUsdc",
            "type": "u64"
          },
          {
            "name": "minTotalAgcOut",
            "type": "u64"
          },
          {
            "name": "agcBurned",
            "type": "u64"
          },
          {
            "name": "maxSliceUsdc",
            "type": "u64"
          },
          {
            "name": "sliceIntervalSeconds",
            "type": "u64"
          },
          {
            "name": "startedAt",
            "type": "u64"
          },
          {
            "name": "expiresAt",
            "type": "u64"
          },
          {
            "name": "lastSliceAt",
            "type": "u64"
          },
          {
            "name": "sliceCount",
            "type": "u64"
          },
          {
            "name": "adapterUsdcAccount",
            "type": "pubkey"
          },
          {
            "name": "usdcEscrow",
            "type": "pubkey"
          },
          {
            "name": "agcVault",
            "type": "pubkey"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "authorityBump",
            "type": "u8"
          },
          {
            "name": "usdcEscrowBump",
            "type": "u8"
          },
          {
            "name": "agcVaultBump",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                128
              ]
            }
          }
        ]
      }
    },
    {
      "name": "buybackCampaignCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "campaign",
            "type": "pubkey"
          },
          {
            "name": "campaignId",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "buybackCampaignConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "totalUsdc",
            "type": "u64"
          },
          {
            "name": "minTotalAgcOut",
            "type": "u64"
          },
          {
            "name": "maxSliceUsdc",
            "type": "u64"
          },
          {
            "name": "sliceIntervalSeconds",
            "type": "u64"
          },
          {
            "name": "startAfter",
            "type": "u64"
          },
          {
            "name": "expiresAt",
            "type": "u64"
          },
          {
            "name": "adapterUsdcAccount",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "buybackCampaignStarted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "campaign",
            "type": "pubkey"
          },
          {
            "name": "campaignId",
            "type": "u64"
          },
          {
            "name": "totalUsdc",
            "type": "u64"
          },
          {
            "name": "minTotalAgcOut",
            "type": "u64"
          },
          {
            "name": "adapterUsdcAccount",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "buybackCampaignStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "uninitialized"
          },
          {
            "name": "active"
          },
          {
            "name": "completed"
          },
          {
            "name": "cancelled"
          }
        ]
      }
    },
    {
      "name": "buybackSliceArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "usdcAmount",
            "type": "u64"
          },
          {
            "name": "agcAmountToBurn",
            "type": "u64"
          },
          {
            "name": "minAgcOut",
            "type": "u64"
          },
          {
            "name": "deadline",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "buybackTwapSliceExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "campaign",
            "type": "pubkey"
          },
          {
            "name": "campaignId",
            "type": "u64"
          },
          {
            "name": "usdcAmount",
            "type": "u64"
          },
          {
            "name": "agcBurned",
            "type": "u64"
          },
          {
            "name": "remainingUsdc",
            "type": "u64"
          },
          {
            "name": "totalAgcBurned",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "buybackUsdcEscrowUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "collateralAsset",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "mintDecimals",
            "type": "u8"
          },
          {
            "name": "oracleSource",
            "type": {
              "defined": {
                "name": "oracleSource"
              }
            }
          },
          {
            "name": "oracleFeed",
            "type": "pubkey"
          },
          {
            "name": "pythPriceFeedId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reserveTokenAccount",
            "type": "pubkey"
          },
          {
            "name": "assetClass",
            "type": {
              "defined": {
                "name": "assetClass"
              }
            }
          },
          {
            "name": "reserveWeightBps",
            "type": "u16"
          },
          {
            "name": "collateralFactorBps",
            "type": "u16"
          },
          {
            "name": "liquidationThresholdBps",
            "type": "u16"
          },
          {
            "name": "maxConcentrationBps",
            "type": "u16"
          },
          {
            "name": "maxOracleStalenessSeconds",
            "type": "u64"
          },
          {
            "name": "maxOracleConfidenceBps",
            "type": "u16"
          },
          {
            "name": "enabled",
            "type": "bool"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "collateralAssetConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "oracleSource",
            "type": {
              "defined": {
                "name": "oracleSource"
              }
            }
          },
          {
            "name": "oracleFeed",
            "type": "pubkey"
          },
          {
            "name": "pythPriceFeedId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reserveTokenAccount",
            "type": "pubkey"
          },
          {
            "name": "assetClass",
            "type": {
              "defined": {
                "name": "assetClass"
              }
            }
          },
          {
            "name": "reserveWeightBps",
            "type": "u16"
          },
          {
            "name": "collateralFactorBps",
            "type": "u16"
          },
          {
            "name": "liquidationThresholdBps",
            "type": "u16"
          },
          {
            "name": "maxConcentrationBps",
            "type": "u16"
          },
          {
            "name": "maxOracleStalenessSeconds",
            "type": "u64"
          },
          {
            "name": "maxOracleConfidenceBps",
            "type": "u16"
          },
          {
            "name": "enabled",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "collateralAssetUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "config",
            "type": {
              "defined": {
                "name": "collateralAssetConfig"
              }
            }
          }
        ]
      }
    },
    {
      "name": "collateralOracle",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "oracleFeed",
            "type": "pubkey"
          },
          {
            "name": "oracleSource",
            "type": {
              "defined": {
                "name": "oracleSource"
              }
            }
          },
          {
            "name": "pythPriceFeedId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "priceQuoteX18",
            "type": "u128"
          },
          {
            "name": "confidenceBps",
            "type": "u16"
          },
          {
            "name": "updatedAt",
            "type": "u64"
          },
          {
            "name": "publishTime",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    },
    {
      "name": "collateralOraclePriceInput",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "priceQuoteX18",
            "type": "u128"
          },
          {
            "name": "confidenceBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "collateralOraclePriceUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "priceQuoteX18",
            "type": "u128"
          },
          {
            "name": "confidenceBps",
            "type": "u16"
          },
          {
            "name": "updatedAt",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditCollateralDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditCollateralWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditFacility",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facilityId",
            "type": "u64"
          },
          {
            "name": "collateralMint",
            "type": "pubkey"
          },
          {
            "name": "collateralAsset",
            "type": "pubkey"
          },
          {
            "name": "collateralVault",
            "type": "pubkey"
          },
          {
            "name": "underwriterVaultAgc",
            "type": "pubkey"
          },
          {
            "name": "collateralDecimals",
            "type": "u8"
          },
          {
            "name": "config",
            "type": {
              "defined": {
                "name": "creditFacilityConfig"
              }
            }
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "creditFacilityStatus"
              }
            }
          },
          {
            "name": "underwriterTotalShares",
            "type": "u64"
          },
          {
            "name": "totalPrincipalDebtAgc",
            "type": "u64"
          },
          {
            "name": "totalUnderwriterDepositsAgc",
            "type": "u128"
          },
          {
            "name": "totalUnderwriterWithdrawalsAgc",
            "type": "u128"
          },
          {
            "name": "totalDrawnAgc",
            "type": "u128"
          },
          {
            "name": "totalRepaidPrincipalAgc",
            "type": "u128"
          },
          {
            "name": "totalInterestAccruedAgc",
            "type": "u128"
          },
          {
            "name": "totalInterestPaidAgc",
            "type": "u128"
          },
          {
            "name": "totalDefaultedAgc",
            "type": "u128"
          },
          {
            "name": "totalUnderwriterLossAgc",
            "type": "u128"
          },
          {
            "name": "totalCollateralDeposited",
            "type": "u128"
          },
          {
            "name": "totalCollateralSeized",
            "type": "u128"
          },
          {
            "name": "activeCreditLines",
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "authorityBump",
            "type": "u8"
          },
          {
            "name": "collateralVaultBump",
            "type": "u8"
          },
          {
            "name": "underwriterVaultBump",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                256
              ]
            }
          }
        ]
      }
    },
    {
      "name": "creditFacilityConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxTotalDebtAgc",
            "type": "u64"
          },
          {
            "name": "maxLineDebtAgc",
            "type": "u64"
          },
          {
            "name": "minCollateralHealthBps",
            "type": "u16"
          },
          {
            "name": "liquidationHealthBps",
            "type": "u16"
          },
          {
            "name": "minUnderwriterReserveBps",
            "type": "u16"
          },
          {
            "name": "interestRateBps",
            "type": "u16"
          },
          {
            "name": "originationFeeBps",
            "type": "u16"
          },
          {
            "name": "defaultGraceSeconds",
            "type": "u64"
          },
          {
            "name": "isolated",
            "type": "bool"
          },
          {
            "name": "enabled",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "creditFacilityConfigUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "config",
            "type": {
              "defined": {
                "name": "creditFacilityConfig"
              }
            }
          }
        ]
      }
    },
    {
      "name": "creditFacilityInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "facilityId",
            "type": "u64"
          },
          {
            "name": "collateralMint",
            "type": "pubkey"
          },
          {
            "name": "config",
            "type": {
              "defined": {
                "name": "creditFacilityConfig"
              }
            }
          }
        ]
      }
    },
    {
      "name": "creditFacilityStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "uninitialized"
          },
          {
            "name": "active"
          },
          {
            "name": "disabled"
          }
        ]
      }
    },
    {
      "name": "creditLine",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "lineId",
            "type": "u64"
          },
          {
            "name": "creditLimitAgc",
            "type": "u64"
          },
          {
            "name": "principalDebtAgc",
            "type": "u64"
          },
          {
            "name": "accruedInterestAgc",
            "type": "u64"
          },
          {
            "name": "collateralAmount",
            "type": "u64"
          },
          {
            "name": "maturityTimestamp",
            "type": "u64"
          },
          {
            "name": "openedAt",
            "type": "u64"
          },
          {
            "name": "lastAccruedAt",
            "type": "u64"
          },
          {
            "name": "defaultedAt",
            "type": "u64"
          },
          {
            "name": "closedAt",
            "type": "u64"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "creditLineStatus"
              }
            }
          },
          {
            "name": "underwriterLossAgc",
            "type": "u64"
          },
          {
            "name": "uncoveredDefaultAgc",
            "type": "u64"
          },
          {
            "name": "collateralSeized",
            "type": "u128"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                128
              ]
            }
          }
        ]
      }
    },
    {
      "name": "creditLineDefaulted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "defaultedDebt",
            "type": "u64"
          },
          {
            "name": "underwriterLoss",
            "type": "u64"
          },
          {
            "name": "uncoveredDebt",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditLineDrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "grossAmount",
            "type": "u64"
          },
          {
            "name": "netAmount",
            "type": "u64"
          },
          {
            "name": "fee",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditLineOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "lineId",
            "type": "u64"
          },
          {
            "name": "creditLimitAgc",
            "type": "u64"
          },
          {
            "name": "maturityTimestamp",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditLineRepaid",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "principalPaid",
            "type": "u64"
          },
          {
            "name": "interestPaid",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditLineStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "uninitialized"
          },
          {
            "name": "active"
          },
          {
            "name": "repaid"
          },
          {
            "name": "defaulted"
          }
        ]
      }
    },
    {
      "name": "defaultedCollateralSeized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "line",
            "type": "pubkey"
          },
          {
            "name": "destination",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "epochAccumulator",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epochId",
            "type": "u64"
          },
          {
            "name": "startedAt",
            "type": "u64"
          },
          {
            "name": "updatedAt",
            "type": "u64"
          },
          {
            "name": "lastObservedAt",
            "type": "u64"
          },
          {
            "name": "observationCount",
            "type": "u64"
          },
          {
            "name": "grossBuyVolumeQuoteX18",
            "type": "u128"
          },
          {
            "name": "grossSellVolumeQuoteX18",
            "type": "u128"
          },
          {
            "name": "totalVolumeQuoteX18",
            "type": "u128"
          },
          {
            "name": "lastMidPriceX18",
            "type": "u128"
          },
          {
            "name": "cumulativeMidPriceTimeX18",
            "type": "u128"
          },
          {
            "name": "cumulativeAbsMidPriceChangeBps",
            "type": "u128"
          },
          {
            "name": "totalHookFeesQuoteX18",
            "type": "u128"
          },
          {
            "name": "totalHookFeesAgc",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "epochResult",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epochId",
            "type": "u64"
          },
          {
            "name": "regime",
            "type": {
              "defined": {
                "name": "regime"
              }
            }
          },
          {
            "name": "anchorPriceX18",
            "type": "u128"
          },
          {
            "name": "anchorNextX18",
            "type": "u128"
          },
          {
            "name": "normalFloorX18",
            "type": "u128"
          },
          {
            "name": "stressedFloorX18",
            "type": "u128"
          },
          {
            "name": "priceTwapX18",
            "type": "u128"
          },
          {
            "name": "premiumBps",
            "type": "u128"
          },
          {
            "name": "premiumPersistenceEpochs",
            "type": "u128"
          },
          {
            "name": "creditOutstandingQuoteX18",
            "type": "u128"
          },
          {
            "name": "grossBuyFloorBps",
            "type": "u128"
          },
          {
            "name": "netBuyPressureBps",
            "type": "u128"
          },
          {
            "name": "buyGrowthBps",
            "type": "u128"
          },
          {
            "name": "exitPressureBps",
            "type": "u128"
          },
          {
            "name": "reserveCoverageBps",
            "type": "u128"
          },
          {
            "name": "stableCashCoverageBps",
            "type": "u128"
          },
          {
            "name": "liquidityDepthCoverageBps",
            "type": "u128"
          },
          {
            "name": "lockedShareBps",
            "type": "u128"
          },
          {
            "name": "lockFlowBps",
            "type": "u128"
          },
          {
            "name": "demandScoreBps",
            "type": "u128"
          },
          {
            "name": "healthScoreBps",
            "type": "u128"
          },
          {
            "name": "mintRateBps",
            "type": "u128"
          },
          {
            "name": "mintBudgetAcp",
            "type": "u128"
          },
          {
            "name": "buybackBudgetQuoteX18",
            "type": "u128"
          },
          {
            "name": "stressScoreBps",
            "type": "u128"
          },
          {
            "name": "grossBuyQuoteX18",
            "type": "u128"
          },
          {
            "name": "grossSellQuoteX18",
            "type": "u128"
          },
          {
            "name": "totalVolumeQuoteX18",
            "type": "u128"
          },
          {
            "name": "depthToTargetSlippageQuoteX18",
            "type": "u128"
          },
          {
            "name": "stableCashReserveQuoteX18",
            "type": "u128"
          },
          {
            "name": "riskWeightedReserveQuoteX18",
            "type": "u128"
          },
          {
            "name": "liquidityDepthQuoteX18",
            "type": "u128"
          },
          {
            "name": "largestCollateralConcentrationBps",
            "type": "u16"
          },
          {
            "name": "oracleConfidenceBps",
            "type": "u16"
          },
          {
            "name": "staleOracleCount",
            "type": "u16"
          },
          {
            "name": "realizedVolatilityBps",
            "type": "u128"
          },
          {
            "name": "xagcDepositsAcp",
            "type": "u128"
          },
          {
            "name": "xagcGrossRedemptionsAcp",
            "type": "u128"
          },
          {
            "name": "treasuryQuoteX18",
            "type": "u128"
          },
          {
            "name": "treasuryAcp",
            "type": "u128"
          },
          {
            "name": "xagcTotalAssetsAcp",
            "type": "u128"
          },
          {
            "name": "mintAllocations",
            "type": {
              "defined": {
                "name": "mintAllocation"
              }
            }
          }
        ]
      }
    },
    {
      "name": "epochSettled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epochId",
            "type": "u64"
          },
          {
            "name": "regime",
            "type": {
              "defined": {
                "name": "regime"
              }
            }
          },
          {
            "name": "anchorNextX18",
            "type": "u128"
          },
          {
            "name": "mintBudgetAcp",
            "type": "u128"
          },
          {
            "name": "buybackBudgetQuoteX18",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "exitFeeUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "exitFeeBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "externalMetrics",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "depthToTargetSlippageQuoteX18",
            "type": "u128"
          },
          {
            "name": "stableCashReserveQuoteX18",
            "type": "u128"
          },
          {
            "name": "riskWeightedReserveQuoteX18",
            "type": "u128"
          },
          {
            "name": "liquidityDepthQuoteX18",
            "type": "u128"
          },
          {
            "name": "largestCollateralConcentrationBps",
            "type": "u16"
          },
          {
            "name": "oracleConfidenceBps",
            "type": "u16"
          },
          {
            "name": "staleOracleCount",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "governanceAuthorities",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "riskAdmin",
            "type": "pubkey"
          },
          {
            "name": "emergencyAdmin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "governanceAuthoritiesUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authorities",
            "type": {
              "defined": {
                "name": "governanceAuthorities"
              }
            }
          }
        ]
      }
    },
    {
      "name": "growthProgramsEnabledUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "enabled",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "initializeProtocolArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "initialAnchorPriceX18",
            "type": "u128"
          },
          {
            "name": "policyParams",
            "type": {
              "defined": {
                "name": "policyParams"
              }
            }
          },
          {
            "name": "mintDistribution",
            "type": {
              "defined": {
                "name": "mintDistribution"
              }
            }
          },
          {
            "name": "settlementRecipients",
            "type": {
              "defined": {
                "name": "settlementRecipients"
              }
            }
          },
          {
            "name": "exitFeeBps",
            "type": "u16"
          },
          {
            "name": "growthProgramsEnabled",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "keeper",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "permissions",
            "type": {
              "defined": {
                "name": "keeperPermissions"
              }
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "keeperPermissions",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "marketReporter",
            "type": "bool"
          },
          {
            "name": "oracleReporter",
            "type": "bool"
          },
          {
            "name": "epochSettler",
            "type": "bool"
          },
          {
            "name": "buybackExecutor",
            "type": "bool"
          },
          {
            "name": "treasuryBurner",
            "type": "bool"
          },
          {
            "name": "creditOperator",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "keeperPermissionsUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "keeper",
            "type": "pubkey"
          },
          {
            "name": "permissions",
            "type": {
              "defined": {
                "name": "keeperPermissions"
              }
            }
          }
        ]
      }
    },
    {
      "name": "marketAdapterAuthorityUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "mintAllocation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "xagcMintAcp",
            "type": "u128"
          },
          {
            "name": "growthProgramsMintAcp",
            "type": "u128"
          },
          {
            "name": "lpMintAcp",
            "type": "u128"
          },
          {
            "name": "integratorsMintAcp",
            "type": "u128"
          },
          {
            "name": "treasuryMintAcp",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "mintDistribution",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "xagcBps",
            "type": "u16"
          },
          {
            "name": "growthProgramsBps",
            "type": "u16"
          },
          {
            "name": "lpBps",
            "type": "u16"
          },
          {
            "name": "integratorsBps",
            "type": "u16"
          },
          {
            "name": "treasuryBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "mintDistributionUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "distribution",
            "type": {
              "defined": {
                "name": "mintDistribution"
              }
            }
          }
        ]
      }
    },
    {
      "name": "openCreditLineArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "creditLimitAgc",
            "type": "u64"
          },
          {
            "name": "maturityTimestamp",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "oracleSource",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "manual"
          },
          {
            "name": "pyth"
          }
        ]
      }
    },
    {
      "name": "pauseFlags",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "xagcDepositsPaused",
            "type": "bool"
          },
          {
            "name": "xagcRedemptionsPaused",
            "type": "bool"
          },
          {
            "name": "marketReportingPaused",
            "type": "bool"
          },
          {
            "name": "settlementPaused",
            "type": "bool"
          },
          {
            "name": "creditIssuancePaused",
            "type": "bool"
          },
          {
            "name": "collateralUpdatesPaused",
            "type": "bool"
          },
          {
            "name": "buybacksPaused",
            "type": "bool"
          },
          {
            "name": "treasuryBurnsPaused",
            "type": "bool"
          },
          {
            "name": "creditFacilityUpdatesPaused",
            "type": "bool"
          },
          {
            "name": "creditLineUpdatesPaused",
            "type": "bool"
          },
          {
            "name": "creditDrawsPaused",
            "type": "bool"
          },
          {
            "name": "creditRepaymentsPaused",
            "type": "bool"
          },
          {
            "name": "underwriterDepositsPaused",
            "type": "bool"
          },
          {
            "name": "underwriterWithdrawalsPaused",
            "type": "bool"
          },
          {
            "name": "liquidationsPaused",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "pauseFlagsUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pauseFlags",
            "type": {
              "defined": {
                "name": "pauseFlags"
              }
            }
          }
        ]
      }
    },
    {
      "name": "policyParametersUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "normalBandBps",
            "type": "u16"
          },
          {
            "name": "stressedBandBps",
            "type": "u16"
          },
          {
            "name": "policyEpochDuration",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "policyParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "normalBandBps",
            "type": "u16"
          },
          {
            "name": "stressedBandBps",
            "type": "u16"
          },
          {
            "name": "anchorEmaBps",
            "type": "u16"
          },
          {
            "name": "maxAnchorCrawlBps",
            "type": "u16"
          },
          {
            "name": "minPremiumBps",
            "type": "u16"
          },
          {
            "name": "premiumPersistenceRequired",
            "type": "u16"
          },
          {
            "name": "minGrossBuyFloorBps",
            "type": "u16"
          },
          {
            "name": "minLockedShareBps",
            "type": "u16"
          },
          {
            "name": "targetGrossBuyBps",
            "type": "u16"
          },
          {
            "name": "targetNetBuyBps",
            "type": "u16"
          },
          {
            "name": "targetLockFlowBps",
            "type": "u16"
          },
          {
            "name": "targetBuyGrowthBps",
            "type": "u16"
          },
          {
            "name": "targetLockedShareBps",
            "type": "u16"
          },
          {
            "name": "expansionReserveCoverageBps",
            "type": "u16"
          },
          {
            "name": "targetReserveCoverageBps",
            "type": "u16"
          },
          {
            "name": "neutralReserveCoverageBps",
            "type": "u16"
          },
          {
            "name": "defenseReserveCoverageBps",
            "type": "u16"
          },
          {
            "name": "hardDefenseReserveCoverageBps",
            "type": "u16"
          },
          {
            "name": "minStableCashCoverageBps",
            "type": "u16"
          },
          {
            "name": "targetStableCashCoverageBps",
            "type": "u16"
          },
          {
            "name": "defenseStableCashCoverageBps",
            "type": "u16"
          },
          {
            "name": "minLiquidityDepthCoverageBps",
            "type": "u16"
          },
          {
            "name": "targetLiquidityDepthCoverageBps",
            "type": "u16"
          },
          {
            "name": "maxReserveConcentrationBps",
            "type": "u16"
          },
          {
            "name": "maxOracleConfidenceBps",
            "type": "u16"
          },
          {
            "name": "maxStaleOracleCount",
            "type": "u16"
          },
          {
            "name": "maxExpansionVolatilityBps",
            "type": "u16"
          },
          {
            "name": "defenseVolatilityBps",
            "type": "u16"
          },
          {
            "name": "maxExpansionExitPressureBps",
            "type": "u16"
          },
          {
            "name": "defenseExitPressureBps",
            "type": "u16"
          },
          {
            "name": "expansionKappaBps",
            "type": "u16"
          },
          {
            "name": "maxMintPerEpochBps",
            "type": "u16"
          },
          {
            "name": "maxMintPerDayBps",
            "type": "u16"
          },
          {
            "name": "buybackKappaBps",
            "type": "u16"
          },
          {
            "name": "mildDefenseSpendBps",
            "type": "u16"
          },
          {
            "name": "severeDefenseSpendBps",
            "type": "u16"
          },
          {
            "name": "severeStressThresholdBps",
            "type": "u16"
          },
          {
            "name": "recoveryCooldownEpochs",
            "type": "u16"
          },
          {
            "name": "policyEpochDuration",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "protocolInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "agcMint",
            "type": "pubkey"
          },
          {
            "name": "xagcMint",
            "type": "pubkey"
          },
          {
            "name": "usdcMint",
            "type": "pubkey"
          },
          {
            "name": "initialAnchorPriceX18",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "protocolState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "pendingAdmin",
            "type": "pubkey"
          },
          {
            "name": "riskAdmin",
            "type": "pubkey"
          },
          {
            "name": "emergencyAdmin",
            "type": "pubkey"
          },
          {
            "name": "agcMint",
            "type": "pubkey"
          },
          {
            "name": "xagcMint",
            "type": "pubkey"
          },
          {
            "name": "usdcMint",
            "type": "pubkey"
          },
          {
            "name": "treasuryAgc",
            "type": "pubkey"
          },
          {
            "name": "treasuryUsdc",
            "type": "pubkey"
          },
          {
            "name": "xagcVaultAgc",
            "type": "pubkey"
          },
          {
            "name": "growthProgramsAgc",
            "type": "pubkey"
          },
          {
            "name": "lpAgc",
            "type": "pubkey"
          },
          {
            "name": "integratorsAgc",
            "type": "pubkey"
          },
          {
            "name": "buybackUsdcEscrow",
            "type": "pubkey"
          },
          {
            "name": "marketAdapterAuthority",
            "type": "pubkey"
          },
          {
            "name": "pythReceiverProgram",
            "type": "pubkey"
          },
          {
            "name": "stateBump",
            "type": "u8"
          },
          {
            "name": "mintAuthorityBump",
            "type": "u8"
          },
          {
            "name": "treasuryAuthorityBump",
            "type": "u8"
          },
          {
            "name": "xagcAuthorityBump",
            "type": "u8"
          },
          {
            "name": "treasuryAgcBump",
            "type": "u8"
          },
          {
            "name": "treasuryUsdcBump",
            "type": "u8"
          },
          {
            "name": "xagcVaultAgcBump",
            "type": "u8"
          },
          {
            "name": "agcDecimals",
            "type": "u8"
          },
          {
            "name": "xagcDecimals",
            "type": "u8"
          },
          {
            "name": "usdcDecimals",
            "type": "u8"
          },
          {
            "name": "agcUnit",
            "type": "u64"
          },
          {
            "name": "quoteScale",
            "type": "u128"
          },
          {
            "name": "exitFeeBps",
            "type": "u16"
          },
          {
            "name": "growthProgramsEnabled",
            "type": "bool"
          },
          {
            "name": "pauseFlags",
            "type": {
              "defined": {
                "name": "pauseFlags"
              }
            }
          },
          {
            "name": "policyParams",
            "type": {
              "defined": {
                "name": "policyParams"
              }
            }
          },
          {
            "name": "mintDistribution",
            "type": {
              "defined": {
                "name": "mintDistribution"
              }
            }
          },
          {
            "name": "regime",
            "type": {
              "defined": {
                "name": "regime"
              }
            }
          },
          {
            "name": "anchorPriceX18",
            "type": "u128"
          },
          {
            "name": "premiumPersistenceEpochs",
            "type": "u128"
          },
          {
            "name": "lastGrossBuyQuoteX18",
            "type": "u128"
          },
          {
            "name": "lastCoverageBps",
            "type": "u128"
          },
          {
            "name": "lastExitPressureBps",
            "type": "u128"
          },
          {
            "name": "lastVolatilityBps",
            "type": "u128"
          },
          {
            "name": "lastPremiumBps",
            "type": "u128"
          },
          {
            "name": "lastLockedShareBps",
            "type": "u128"
          },
          {
            "name": "lastLockFlowBps",
            "type": "u128"
          },
          {
            "name": "lastStableCashCoverageBps",
            "type": "u128"
          },
          {
            "name": "lastLiquidityDepthCoverageBps",
            "type": "u128"
          },
          {
            "name": "lastReserveConcentrationBps",
            "type": "u128"
          },
          {
            "name": "lastOracleConfidenceBps",
            "type": "u128"
          },
          {
            "name": "lastStaleOracleCount",
            "type": "u16"
          },
          {
            "name": "lastSettledEpoch",
            "type": "u64"
          },
          {
            "name": "lastSettlementTimestamp",
            "type": "u64"
          },
          {
            "name": "recoveryCooldownEpochsRemaining",
            "type": "u64"
          },
          {
            "name": "mintWindowDay",
            "type": "u64"
          },
          {
            "name": "mintedInCurrentDay",
            "type": "u128"
          },
          {
            "name": "pendingTreasuryBuybackUsdc",
            "type": "u64"
          },
          {
            "name": "xagcGrossDepositsTotal",
            "type": "u128"
          },
          {
            "name": "xagcGrossRedemptionsTotal",
            "type": "u128"
          },
          {
            "name": "xagcUnaccountedAssets",
            "type": "u64"
          },
          {
            "name": "lastXagcDepositTotal",
            "type": "u128"
          },
          {
            "name": "lastXagcRedemptionTotal",
            "type": "u128"
          },
          {
            "name": "buybackExecutionNonce",
            "type": "u64"
          },
          {
            "name": "protocolVersion",
            "type": "u16"
          },
          {
            "name": "creditFacilityCount",
            "type": "u64"
          },
          {
            "name": "creditPrincipalOutstandingAgc",
            "type": "u128"
          },
          {
            "name": "creditDrawnAgc",
            "type": "u128"
          },
          {
            "name": "creditRepaidAgc",
            "type": "u128"
          },
          {
            "name": "creditInterestPaidAgc",
            "type": "u128"
          },
          {
            "name": "creditDefaultedAgc",
            "type": "u128"
          },
          {
            "name": "accumulator",
            "type": {
              "defined": {
                "name": "epochAccumulator"
              }
            }
          },
          {
            "name": "lastEpochResult",
            "type": {
              "defined": {
                "name": "epochResult"
              }
            }
          }
        ]
      }
    },
    {
      "name": "pythReceiverProgramUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "receiverProgram",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "recordSwapArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "agcAmount",
            "type": "u64"
          },
          {
            "name": "usdcAmount",
            "type": "u64"
          },
          {
            "name": "priceX18",
            "type": "u128"
          },
          {
            "name": "agcToUsdc",
            "type": "bool"
          },
          {
            "name": "hookFeeUsdc",
            "type": "u64"
          },
          {
            "name": "hookFeeAgc",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "regime",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "neutral"
          },
          {
            "name": "expansion"
          },
          {
            "name": "defense"
          },
          {
            "name": "recovery"
          }
        ]
      }
    },
    {
      "name": "settlementRecipients",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "growthProgramsAgc",
            "type": "pubkey"
          },
          {
            "name": "lpAgc",
            "type": "pubkey"
          },
          {
            "name": "integratorsAgc",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "settlementRecipientsUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "recipients",
            "type": {
              "defined": {
                "name": "settlementRecipients"
              }
            }
          }
        ]
      }
    },
    {
      "name": "swapRecorded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epochId",
            "type": "u64"
          },
          {
            "name": "agcAmount",
            "type": "u64"
          },
          {
            "name": "usdcAmount",
            "type": "u64"
          },
          {
            "name": "priceX18",
            "type": "u128"
          },
          {
            "name": "agcToUsdc",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "treasuryAgcBurned",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "treasuryBuybackUsdcReserved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "usdcSpent",
            "type": "u64"
          },
          {
            "name": "pendingTreasuryBuybackUsdcAfter",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "underwriterAgcDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "underwriter",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "shares",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "underwriterAgcWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "underwriter",
            "type": "pubkey"
          },
          {
            "name": "assets",
            "type": "u64"
          },
          {
            "name": "shares",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "underwriterPosition",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "facility",
            "type": "pubkey"
          },
          {
            "name": "underwriter",
            "type": "pubkey"
          },
          {
            "name": "shares",
            "type": "u64"
          },
          {
            "name": "depositedAgc",
            "type": "u128"
          },
          {
            "name": "withdrawnAgc",
            "type": "u128"
          },
          {
            "name": "lossAgc",
            "type": "u128"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                128
              ]
            }
          }
        ]
      }
    },
    {
      "name": "xagcDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "caller",
            "type": "pubkey"
          },
          {
            "name": "receiverXagc",
            "type": "pubkey"
          },
          {
            "name": "assets",
            "type": "u64"
          },
          {
            "name": "shares",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "xagcRedeemed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "caller",
            "type": "pubkey"
          },
          {
            "name": "receiverAgc",
            "type": "pubkey"
          },
          {
            "name": "shares",
            "type": "u64"
          },
          {
            "name": "grossAssets",
            "type": "u64"
          },
          {
            "name": "feeAssets",
            "type": "u64"
          },
          {
            "name": "netAssets",
            "type": "u64"
          }
        ]
      }
    }
  ]
};
