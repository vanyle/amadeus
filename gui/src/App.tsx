import {
  Button,
  FormControl,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  TextField,
} from "@material-ui/core";
import { Autocomplete } from "@mui/material";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFnsV3";
import { DatePicker } from "@mui/x-date-pickers/DatePicker/DatePicker";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider/LocalizationProvider";
import React, { useMemo } from "react";
import {
  VictoryAxis,
  VictoryChart,
  VictoryLegend,
  VictoryLine,
  VictoryTheme,
} from "victory";
import "./App.css";
import { airports } from "./airports";

function App() {
  const data = useMemo(() => {
    return {
      labels: ["January", "February", "March", "April", "May", "June", "July"],
      datasets: [
        {
          label: "Revenue",
          data: [65, 59, 80, 81, 56, 55, 40],
          fill: false,
          borderColor: "rgb(75, 192, 192)",
          tension: 0.1,
        },
      ],
    };
  }, []);

  const handleChange = () => {
    console.log("change");
  };
  const age = 10;

  return (
    <div className="flex flex-col items-center">
      <h1 className="text-amadeus font-bold">Amadeus analytics</h1>
      <Paper className="p-6 flex flex-col items-start">
        <h2 className="text-2xl text-left font-bold">
          Search for flight prices
        </h2>
        <div className="h-4" />
        <div className="flex justify-center">
          <FormControl className="w-[100px]">
            <InputLabel>Trip type</InputLabel>
            <Select
              labelId="demo-simple-select-label"
              id="demo-simple-select"
              value={age}
              label="Age"
              onChange={handleChange}
            >
              <MenuItem value={10}>One way</MenuItem>
              <MenuItem value={20}>Round trip</MenuItem>
            </Select>
          </FormControl>
          <div className="w-4 inline-block" />
          <Autocomplete
            className="inline"
            options={airports}
            sx={{ width: 300 }}
            renderInput={(params) => (
              <TextField {...params} variant="outlined" label="Where from?" />
            )}
          />
          <div className="w-4 inline-block" />

          <Autocomplete
            options={airports}
            sx={{ width: 300 }}
            renderInput={(params) => (
              <TextField {...params} variant="outlined" label="Where to?" />
            )}
          />
          <div className="w-4 inline-block" />

          <LocalizationProvider dateAdapter={AdapterDateFns}>
            <DatePicker label="Search for trips after " />
            <div className="w-4 inline-block" />

            <DatePicker label="Search for trips before" />
          </LocalizationProvider>
        </div>
        <div className="h-4" />
        <div className="text-left">
          <Button variant="contained" color="primary">
            Explore
          </Button>
        </div>
      </Paper>

      <div className="w-[100%] flex flex-col">
        <div className="text-xl text-left p-5">
          Cheapest median price: {100}€
        </div>
        <VictoryChart width={700} height={400} theme={VictoryTheme.material}>
          <VictoryLegend
            x={0}
            y={0}
            centerTitle
            orientation="horizontal"
            gutter={20}
            style={{ border: { stroke: "black" }, title: { fontSize: 20 } }}
            data={[
              { name: "Airfrance", symbol: { fill: "tomato" } },
              { name: "Easy-Jet", symbol: { fill: "orange" } },
              { name: "Delta", symbol: { fill: "gold" } },
            ]}
          />
          <VictoryLine
            style={{
              data: { stroke: "#c43a31" },
              parent: { border: "1px solid #ccc" },
            }}
            data={[
              { x: 1, y: 2 },
              { x: 2, y: 3 },
              { x: 3, y: 5 },
              { x: 4, y: 4 },
              { x: 5, y: 7 },
            ]}
          />
          <VictoryAxis
            label="Advance purchase (in days)"
            style={{
              tickLabels: { fontSize: 15, padding: 5 },
              ticks: { stroke: "grey", size: 5 },
              axisLabel: { fontSize: 20, padding: 30 },
            }}
          />
          <VictoryAxis
            dependentAxis
            label="Ticket price (in euros)"
            style={{ axisLabel: { fontSize: 15, padding: 30 } }}
          />
        </VictoryChart>
      </div>
    </div>
  );
}

export default App;
