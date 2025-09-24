import { ComputeBudgetProgram, Connection, Keypair, sendAndConfirmTransaction, TransactionInstruction } from "@solana/web3.js"
import { Transaction } from "@solana/web3.js"
import base58 from "bs58"
import { createBurnCheckedInstruction, createCloseAccountInstruction, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { SPL_ACCOUNT_LAYOUT, TokenAccount } from "@raydium-io/raydium-sdk"

import dotenv from 'dotenv';

dotenv.config();

const rpc = process.env.VITE_RPC_ENDPOINT;
const privateKey = process.env.PK;

let connection = null

if (rpc) {
    connection = new Connection(rpc);
}

const burnAllTokens = async (pk: string) => {
    try {
        const mainKp = Keypair.fromSecretKey(base58.decode(pk))

        if (!connection) return

        const tokenAccounts = await connection.getTokenAccountsByOwner(mainKp.publicKey, {
            programId: TOKEN_PROGRAM_ID,
        },
            "confirmed"
        )
        const ixs: TransactionInstruction[] = []
        const accounts: TokenAccount[] = [];

        if (tokenAccounts.value.length > 0)
            for (const { pubkey, account } of tokenAccounts.value) {
                accounts.push({
                    pubkey,
                    programId: account.owner,
                    accountInfo: SPL_ACCOUNT_LAYOUT.decode(account.data),
                });
            }

        console.log('account list')

        for (let j = 0; j < accounts.length; j++) {
            const tokenAccount = accounts[j].pubkey
            const tokenBalance = (await connection.getTokenAccountBalance(accounts[j].pubkey)).value
            if (tokenBalance.uiAmount && tokenBalance.uiAmount > 0){
                // console.log("account mint => ", accounts[j].accountInfo.mint.toString())
                if(accounts[j].accountInfo.mint.toString() === 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v') continue;
                ixs.push(createBurnCheckedInstruction(tokenAccount, accounts[j].accountInfo.mint, mainKp.publicKey, BigInt(tokenBalance.amount), tokenBalance.decimals))
            }
            ixs.push(createCloseAccountInstruction(tokenAccount, mainKp.publicKey, mainKp.publicKey))
        }

        if (ixs.length) {
            const tx = new Transaction().add(
                ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 220_000 }),
                ComputeBudgetProgram.setComputeUnitLimit({ units: 350_000 }),
                ...ixs.slice(0, 22),
            )
            tx.feePayer = mainKp.publicKey
            tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
            console.log(await connection.simulateTransaction(tx))
            const sig = await sendAndConfirmTransaction(connection, tx, [mainKp], { commitment: "confirmed" })
            console.log(`Closed and gathered SOL from wallets  : https://solscan.io/tx/${sig}`)
            return
        }
    } catch (error) {
        console.log("🚀 ~ burn ~ error:", error)

    }
}

if(privateKey)
    burnAllTokens(privateKey);
