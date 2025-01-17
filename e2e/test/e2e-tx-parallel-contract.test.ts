import { expect } from "chai";

import { TestContractBalances, TestContractCounter } from "../typechain-types";
import { ALICE, BOB, CHARLIE, randomAccounts } from "./helpers/account";
import {
    deployTestContractBalances,
    deployTestContractCounter,
    sendGetNonce,
    sendRawTransaction,
    sendRawTransactions,
    sendReset,
} from "./helpers/rpc";

describe("Transaction: parallel TestContractBalances", async () => {
    var _contract: TestContractBalances;

    it("Resets blockchain", async () => {
        await sendReset();
    });

    it("Deploy TestContractBalances", async () => {
        _contract = await deployTestContractBalances();
    });

    it("Sends parallel transactions", async () => {
        // initial balance
        expect(await _contract.get(ALICE.address)).eq(0);
        expect(await _contract.get(BOB.address)).eq(0);
        expect(await _contract.get(CHARLIE.address)).eq(0);

        // prepare transactions
        const expectedBalances: Record<string, number> = {};
        expectedBalances[ALICE.address] = 0;
        expectedBalances[BOB.address] = 0;
        expectedBalances[CHARLIE.address] = 0;

        const senders = randomAccounts(50);
        const signedTxs = [];
        for (let accountIndex = 0; accountIndex < senders.length; accountIndex++) {
            // prepare transaction params
            let account = ALICE.address;
            if (accountIndex % 2 == 0) {
                account = BOB.address;
            } else if (accountIndex % 3 == 0) {
                account = CHARLIE.address;
            }
            const amount = accountIndex + 1;
            expectedBalances[account] += amount;

            // sign transaction
            const sender = senders[accountIndex];
            const nonce = await sendGetNonce(sender.address);
            const tx = await _contract
                .connect(sender.signer())
                .add.populateTransaction(account, amount, { nonce: nonce, gasPrice: 0 });
            signedTxs.push(await sender.signer().signTransaction(tx));
        }

        // send transactions in parallel
        await sendRawTransactions(signedTxs);

        // verify
        expect(await _contract.get(ALICE.address)).eq(expectedBalances[ALICE.address]);
        expect(await _contract.get(BOB.address)).eq(expectedBalances[BOB.address]);
        expect(await _contract.get(CHARLIE.address)).eq(expectedBalances[CHARLIE.address]);
    });
});

describe("Transaction: parallel TestContractCounter", async () => {
    var _contract: TestContractCounter;

    it("Resets blockchain", async () => {
        await sendReset();
    });

    it("Deploy TestContractCounter", async () => {
        _contract = await deployTestContractCounter();
    });

    it("Sends parallel transactions", async () => {
        // initial balance
        expect(await _contract.getCounter()).eq(0);
        expect(await _contract.getDoubleCounter()).eq(0);

        const incSender = ALICE;
        const doubleSender = BOB;

        // send pair of inc and double requests
        for (let i = 0; i < 20; i++) {
            // calculate expected double counter
            const doubleCounter = Number(await _contract.getDoubleCounter());
            const expectedDoubleCounter = [BigInt(doubleCounter + i * 2), BigInt(doubleCounter + (i + 1) * 2)];

            // sign transactions
            const incNonce = await sendGetNonce(incSender.address);
            const incTx = await _contract
                .connect(incSender.signer())
                .inc.populateTransaction({ nonce: incNonce, gasPrice: 0 });
            const incSignedTx = await incSender.signer().signTransaction(incTx);

            const doubleNonce = await sendGetNonce(doubleSender.address);
            const doubleTx = await _contract
                .connect(doubleSender.signer())
                .double.populateTransaction({ nonce: doubleNonce, gasPrice: 0 });
            const doubleSignedTx = await doubleSender.signer().signTransaction(doubleTx);

            // send transactions in parallel
            await sendRawTransactions([incSignedTx, doubleSignedTx]);

            // verify
            expect(await _contract.getCounter()).eq(i + 1);
            expect(await _contract.getDoubleCounter()).oneOf(expectedDoubleCounter);
        }
    });
});
