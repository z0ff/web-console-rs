function wait(t) {
    return new Promise(resolve => setTimeout(resolve, t));
}

async function getStatus() {
    var getreq = new XMLHttpRequest();
    getreq.open('GET', 'https://127.0.0.1:8080/status');
    getreq.send();

    getreq.onreadystatechange = function () {
        if (getreq.readyState != 4) {
            console.log("readyState != 4");
        } else if (getreq.status != 200) {
            console.log("failed");
        } else {
            var statusData = JSON.parse(getreq.responseText);
            var elem = document.getElementById("osType");
            elem.innerText = statusData.os_type;
            var elem = document.getElementById("osRelease");
            elem.innerText = statusData.os_release;
            var elem = document.getElementById("cpuNum");
            elem.innerText = statusData.cpu_num;
            var elem = document.getElementById("cpuSpeed");
            elem.innerText = statusData.cpu_speed;
            var elem = document.getElementById("cpuProcTotal");
            elem.innerText = statusData.proc_total;
            var elem = document.getElementById("cpuUser");
            elem.innerText = statusData.cpu_user;
            var elem = document.getElementById("cpuNice");
            elem.innerText = statusData.cpu_nice;
            var elem = document.getElementById("cpuSystem");
            elem.innerText = statusData.cpu_system;
            var elem = document.getElementById("cpuIdle");
            elem.innerText = statusData.cpu_idle;
            var elem = document.getElementById("cpuLoadOne");
            elem.innerText = statusData.load_one;
            var elem = document.getElementById("cpuLoadFive");
            elem.innerText = statusData.load_five;
            var elem = document.getElementById("cpuLoadFifteen");
            elem.innerText = statusData.load_fifteen;
            var elem = document.getElementById("memTotal");
            elem.innerText = statusData.mem_total;
            var elem = document.getElementById("memFree");
            elem.innerText = statusData.mem_free;

            cpuChart.data.datasets[0].data[0] = statusData.cpu_user;
            cpuChart.data.datasets[0].data[1] = statusData.cpu_nice;
            cpuChart.data.datasets[0].data[2] = statusData.cpu_system;
            cpuChart.data.datasets[0].data[3] = statusData.cpu_idle;
            cpuChart.update();

            memChart.data.datasets[0].data[1] = statusData.mem_free;
            memChart.data.datasets[0].data[0] = statusData.mem_total - statusData.mem_free;
            memChart.update();

            var elem = document.getElementById("cpuUsage");
            elem.innerText = "CPU Usage: " + (Math.round(10000 - statusData.cpu_idle * 10000) / 100) + "%";
            var elem = document.getElementById("memUsage");
            elem.innerText = "Memory Usage: " + (Math.round((statusData.mem_total - statusData.mem_free) / 10000000) / 100) + "GB /" + (Math.round(statusData.mem_total / 10000000) / 100) + "GB";

        }
    }

    await wait(5000);
    getStatus();
}

document.getElementById('sendScript').onClick = function() {
    //var emailHash = await digestMessage(inputEmail.value.toString());

    var lines = inputScript.value.toString();
    //console.log(lines);
    var postData = {
        //'user': emailHash,
        'lines': lines
    };
    //console.log(postData);
    var postTxt = JSON.stringify(postData);
    //console.log(postTxt);

    if (!isEstalbished) {
        console.log("Connection is not estalbished.");
        return;
    }

    //webSocket.send(postTxt);
    webSocket.send(postTxt);
    /*
    var postreq = new XMLHttpRequest();
    postreq.open('POST', 'https://127.0.0.1:8080/script/postscript');
    postreq.setRequestHeader('content-type', 'application/json');
    postreq.send(postTxt);
    */
}

document.getElementById('clearOutput').onClick = function() {
    output.value = "";
}

var isEstalbished = false;
webSocket = new WebSocket((window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/script/ws/');
webSocket.onopen = function (event) {
    isEstalbished = true;
}
//let inputEmail = document.getElementById('email')
let inputScript = document.getElementById('input');
let output = document.getElementById('output');
webSocket.onmessage = function (event) {
    console.log(event.data);
    output.value += event.data + '\n';
    output.scrollTop = output.scrollHeight;
}

let memUsedVar = 0;
let memFreeVar = 0;

let cpuChartCtx = document.getElementById('cpuUsageChart');
let cpuChart = new Chart(cpuChartCtx, {
    type: 'doughnut',
    data: {
        labels: ["User", "Nice", "System", "Idle"],
        datasets: [{
            backgroundColor: [
                "#00fa9a",
                "#ff8c00",
                "#8a2be2",
                "#a9a9a9"
            ],
            data: [0, 0, 0, 0]
        }]
    },
    options: {
        responsive: false,
        cutoutPercentage: 50,
        plugins: {
            title: {
                display: true,
                text: 'CPU Usage',
                font: {
                    size: 20
                }
            },
            legend: {
                position: 'right'
            }
        }
    }
});

let memChartCtx = document.getElementById('memUsageChart');
let memChart = new Chart(memChartCtx, {
    type: 'doughnut',
    data: {
        labels: ["Used", "Free"],
        datasets: [{
            backgroundColor: [
                "#00bfff",
                "#a9a9a9"
            ],
            data: [0, 0]
        }]
    },
    options: {
        responsive: false,
        cutoutPercentage: 50,
        plugins: {
            title: {
                display: true,
                text: 'Memory Usage',
                font: {
                    size: 20
                }
            },
            legend: {
                position: 'right'
            }
        }
    }
});

getStatus();