import * as wasm from "./pkg/insights2.js";

wasm.default("./pkg/insights2_bg.wasm").then(function() {
  const state = wasm.State.new();
  state.draw_dist();
  state.draw_insights();
  state.draw_timeline();

  // Progress distribution
  const prog_dist = document.getElementById('progress_dist');

  const num_samples = document.getElementById('num_samples');
  num_samples.addEventListener("change", function() {
    state.set_num_samples(num_samples.value);
  });

  const reset_btn = document.getElementById('reset');
  const pareto_btn = document.getElementById('pareto_btn');
  const pareto_uni_btn = document.getElementById('uniform_pareto_btn');
  const pareto_beta_btn = document.getElementById('beta_pareto_btn');

  const min_insights = document.getElementById('min_insights');
  const pareto_q = document.getElementById('pareto_q');
  const alpha = document.getElementById('alpha');
  const beta = document.getElementById('beta');

  pareto_btn.addEventListener("click", function () {
    state.set_pareto(min_insights.value, pareto_q.value);
    state.draw_dist();
    state.draw_timeline();
  });

  pareto_uni_btn.addEventListener("click", function () {
    state.set_pareto_uniform(min_insights.value)
    state.draw_dist();
    state.draw_timeline();
  });

  pareto_beta_btn.addEventListener("click", function () {
    state.set_pareto_beta(min_insights.value, alpha.value, beta.value);
    state.draw_dist();
    state.draw_timeline();
  });

  reset_btn.addEventListener("click", function () {
    state.reset_progress();
  });

  prog_dist.addEventListener("click", add_point);

  function add_point(e) {
    let x = e.offsetX;
    let y = e.offsetY;

    state.add_point(x, y);
    state.draw_dist();
    state.draw_timeline();
  }

  // Insights 
  const year_slider = document.getElementById('year_slider');
  const range = document.getElementById('year_range');
  const reg = document.getElementById('regression');

  reg.addEventListener("change", function() {
    state.set_mode(reg.value);
    state.draw_insights();
    state.draw_timeline();
  });

  noUiSlider.create(year_slider, {
    start: [1945, 2016],
    connect: true,
    range: {
      'min': -600,
      '45%': 250,
      '50%': 1650,
      'max': 2016
    }
  });

  year_slider.noUiSlider.on('update', update_insights);

  function update_insights() {
    let [start, end] = year_slider.noUiSlider.get();

    [start, end] = [Number(start).toFixed(0), Number(end).toFixed(0)];
    range.innerText = `${start} - ${end}`;

    state.set_year_range(start, end);
    state.draw_insights();
    state.draw_timeline();
  }

  const last = document.getElementById("last_year")
  noUiSlider.create(last, {
    start: 2200,
    range: {
      'min': 2050,
      'max': 3500,
    }
  }
  );

  last.noUiSlider.on('update', function() {
    let val = last.noUiSlider.get();
    state.set_last(val);
    state.draw_timeline();
  });


});
