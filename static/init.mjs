export default function myInitializer () {
    return {
        onStart: () => {
            console.log("Loading...");
            console.time("trunk-initializer");
        },
        onProgress: ({current, total}) => {
            if (!total) {
                console.log("Loading...", current, "bytes");
            } else {
                let percentLoaded = Math.round((current/total) * 100);
                console.log("Loading...", percentLoaded, "%" );
                document.getElementById("message").innerHTML = "Loading... " + percentLoaded + "%";
            }
        },
        onComplete: () => {
            console.log("Loading... done!");
            console.timeEnd("trunk-initializer");
            document.getElementById("message").innerHTML = "";
        },
        onSuccess: (wasm) => {
            console.log("Loading... successful!");
            console.log("WebAssembly: ", wasm);
        },
        onFailure: (error) => {
            console.warn("Loading... failed!", error);
        }
    }
};