import { Worker, NearAccount, tGas, NEAR, BN } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async t => {
    const worker = await Worker.init();
    const root = worker.rootAccount;
    const nft = await root.createAndDeploy(
        'non-fungible-token',
        '../../res/non_fungible_token.wasm',
        {
            method: "new_default_meta",
            args: { owner_id: root },
        }
    );
    await root.call(
        nft,
        "nft_mint",
        {
            token_id: "0",
            receiver_id: root,
            token_metadata: {
                title: "Olympus Mons",
                description: "The tallest mountain in the charted solar system",
                media: null,
                media_hash: null,
                copies: 10000,
                issued_at: null,
                expires_at: null,
                starts_at: null,
                updated_at: null,
                extra: null,
                reference: null,
                reference_hash: null,
            }
        },
        { attachedDeposit: '7000000000000000000000' }
    );

    const alice = await root.createSubAccount('alice', { initialBalance: NEAR.parse('100 N').toJSON() });

    const tokenReceiver = await root.createAndDeploy(
        'token-receiver',
        '../../res/token_receiver.wasm',
        {
            method: "new",
            args: { non_fungible_token_account_id: nft },
        }
    );
    const approvalReceiver = await root.createAndDeploy(
        'approval-receiver',
        '../../res/approval_receiver.wasm',
        {
            method: "new",
            args: { non_fungible_token_account_id: nft },
        }
    );

    t.context.worker = worker;
    t.context.accounts = { root, alice, nft, tokenReceiver, approvalReceiver };
});

test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});


test('Simple approve', async test => {
    const { root, alice, nft, tokenReceiver } = test.context.accounts;
    // root approves alice
    await root.call(
        nft,
        'nft_approve',
        {
            token_id: '0',
            account_id: alice,
        },
        { 
            attachedDeposit: new BN('270000000000000000000'), // need more deposit than the sim-tests, cause names are longer
            gas: tGas('150') 
        },
    );

    // check nft_is_approved, don't provide approval_id
    test.assert(
        await nft.view(
            'nft_is_approved',
            {
                token_id: '0',
                approved_account_id: alice,
            })
    );

    // check nft_is_approved, with approval_id=1
    test.assert(
        await nft.view(
            'nft_is_approved',
            {
                token_id: '0',
                approved_account_id: alice,
                approval_id: 1,
            })
    );

    // check nft_is_approved, with approval_id=2
    test.false(
        await nft.view(
            'nft_is_approved',
            {
                token_id: '0',
                approved_account_id: alice,
                approval_id: 2,
            })
    );

    // alternatively, one could check the data returned by nft_token
    const token: any = await nft.view('nft_token', { token_id: '0', });
    test.deepEqual(token.approved_account_ids, { [alice.accountId]: 1 })

    // root approves alice again, which changes the approval_id and doesn't require as much deposit
    await root.call(
        nft,
        'nft_approve',
        {
            token_id: '0',
            account_id: alice,
        },
        { attachedDeposit: '1', gas: tGas('150') }
    );

    test.true(
        await nft.view(
            'nft_is_approved',
            {
                token_id: '0',
                approved_account_id: alice,
                approval_id: 2,
            },
        )
    );

    // approving another account gives different approval_id
    await root.call(
        nft,
        'nft_approve',
        {
            token_id: '0',
            account_id: tokenReceiver,
        },
        // note that token_receiver's account name is longer, and so takes more bytes to store and
        // therefore requires a larger deposit!
        { attachedDeposit: new BN('360000000000000000000'), gas: tGas('150') }
    );

    test.true(
        await nft.view(
            'nft_is_approved',
            {
                token_id: '0',
                approved_account_id: tokenReceiver,
                approval_id: 3,
            })
    );
});

test('Approved account transfers token', async test => {
    const { root, alice, nft } = test.context.accounts;
    await root.call(
        nft,
        'nft_approve',
        {
            token_id: '0',
            account_id: alice,

        },
        { attachedDeposit: new BN('270000000000000000000'), gas: tGas('150') },
    );

    await alice.call(
        nft,
        'nft_transfer',
        {
            receiver_id: alice,
            token_id: '0',
            approval_id: 1,
            memo: 'gotcha! bahahaha',
        },
        { attachedDeposit: '1', gas: tGas('150') }
    );

    const token: any = await nft.view('nft_token', { token_id: '0' });
    test.is(token.owner_id, alice.accountId);
});