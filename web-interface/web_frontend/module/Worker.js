self.onmessage = function(event) {
    animHandler = event.data;

    while (true) {
        console.log("Hello World!");
        animHandler.nextFrame();
    }
}