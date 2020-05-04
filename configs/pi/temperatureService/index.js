const tempRegex = /\d{2}\.\d/;
const timeRegex = /\d{10}/;

const parseLog = (content) => {
  const timeRange = [];
  const tempRange = [];
  content.split("\n").forEach((line) => {
    const time = line.match(timeRegex);
    const temp = line.match(tempRegex);
    time && timeRange.push(time[0]);
    temp && tempRange.push(temp[0]);
  });
  return [timeRange, tempRange];
};

let data = [
  [1546300800, 1546387200], // x-values (timestamps)
  [35, 71], // y-values (series 1)
  [90, 15], // y-values (series 2)
];

const readLog = async () => {
  const reader = new FileReader();
  const file = await fetch("http://localhost:5000/temperature.log");
  const content = await file.blob();

  reader.readAsText(content, "UTF-8");
  reader.onload = (evt) => {
    data = parseLog(evt.target.result);
    new uPlot(opts, data, document.body);
  };
};

readLog();

let opts = {
  title: "Pi Temperature",
  id: "chart1",
  class: "my-chart",
  width: 800,
  height: 600,
  series: [
    {},
    {
      // initial toggled state (optional)
      show: true,
      spanGaps: false,
      // in-legend display
      label: "Temp",
      value: (self, rawValue) => rawValue + "C",

      // series style
      stroke: "red",
      width: 1,
      fill: "rgba(255, 0, 0, 0.3)",
      dash: [10, 5],
    },
  ],
};
