import { ApiPromise, WsProvider } from "@polkadot/api";

async function main() {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: {
      NativeOrAssetId: {
        _enum: {
          Native: null,
          Asset: "u32",
        },
      },
    },
  });

  console.log("Running checks...");
  for(const section of Object.keys(api.query)) {
    for(const method of Object.keys(api.query[section])) {
      if(api.query[section][method].entries instanceof Function) {
	      await api.query[section][method].entries();
      } else {
	      await api.query[section][method]();
      }
    }
  }
  console.log("All OK!");
}

main()
  .catch((e) => {
    console.error(e);
    console.error("Problem found, missing migration?");
    process.exit(1);
  })
  .finally(() => process.exit());
