const anchor = require('@project-serum/anchor');

describe('test1', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it('Is initialized!', async () => {
  
    // Add your test here.
    const program = anchor.workspace.Test1;
    await program.rpc.initialize();
    const owner = new anchor.web3.Account();
    let tx=await program.rpc.seyHello({
      accounts: {owner:owner.publicKey,programId:new anchor.web3.PublicKey("7oMe8iT7TgVXck4AiNUjcoUEF4KiWP665GZJQFT8csMF")}
    })
    console.log("Your transaction signature", tx);
  });
});
