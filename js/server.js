const http = require('http');

const PORT = 8000;
const HOST = 'localhost';

let localServer = http.createServer(requestListener);

function requestListener(request, response) {
    console.log(request.rawHeaders)

    response.writeHead(200);
    response.end("My first server!");
};

function run() {
    localServer.listen(PORT, HOST);
};

function stop() {
    localServer.close()
};

module.exports = {
    run,
    stop
}