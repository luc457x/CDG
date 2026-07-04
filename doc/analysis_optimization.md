# Data Processing & Portfolio Optimization

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

This document outlines the data alignment strategies, technical indicator formulas, machine learning pre-processing pipelines, and Markowitz portfolio optimization mechanics in **CryptoDataGather (CDG)**.

---

## 1. Weekend Gap Alignment

Cryptocurrencies trade 24/7/365, while stock benchmarks close on weekends. When joining datasets, CDG aligns timestamps using two strategies:

### A. Forward-Fill (Default)
Any weekend missing values (e.g. S&P 500 prices on Saturday and Sunday) are populated using the preceding Friday's closing price.
- **Benefits**: Retains weekend crypto data points while keeping stock indexes correctly matched to weekdays.
- **Rust Implementation**: Leverages Polars `fill_null(FillNullStrategy::Forward(None))` and then `fill_null(FillNullStrategy::Backward(None))` for boundary conditions.

### B. Drop Weekends (`--drop-weekends`)
Any rows falling on Saturday or Sunday are discarded from the aligned DataFrame.
- **Benefits**: Simplifies time-series analysis for traditional markets.
- **Rust Implementation**: Uses Chrono `NaiveDate::weekday()` to check for Saturday/Sunday and filters out those records.

---

## 2. Technical Indicators Formulas

CDG computes high-performance technical indicators on the aligned DataFrame using Polars expressions.

### Returns
- **Simple Return**:
  $$R_t = \left( \frac{P_t}{P_{t-1}} - 1 \right) \times 100$$
- **Log Return**:
  $$\ln R_t = \ln\left(\frac{P_t}{P_{t-1}}\right) \times 100$$

### Moving Averages & Bands
- **Simple Moving Average (SMA)**:
  $$\text{SMA}_t = \frac{1}{N}\sum_{i=0}^{N-1} P_{t-i}$$
- **Exponential Moving Average (EMA)**:
  $$\text{EMA}_t = P_t \times k + \text{EMA}_{t-1} \times (1 - k), \quad \text{where } k = \frac{2}{N+1}$$
- **Bollinger Bands**:
  $$\text{Upper Band}_t = \text{SMA}_t + (k \times \sigma_t)$$
  $$\text{Lower Band}_t = \text{SMA}_t - (k \times \sigma_t)$$
  *(where $\sigma_t$ is the standard deviation over the rolling window, default $N=20$, $k=2.0$)*

### Oscillators & Volatility
- **Relative Strength Index (RSI)**:
  $$\text{RSI}_t = 100 - \frac{100}{1 + \text{RS}}, \quad \text{where } \text{RS} = \frac{\text{Smoothed Gain}}{\text{Smoothed Loss}}$$
  *Uses Wilder's smoothing method over a default 14-period window.*
- **MACD (Moving Average Convergence Divergence)**:
  $$\text{MACD Line}_t = \text{EMA}(12)_t - \text{EMA}(26)_t$$
  $$\text{Signal Line}_t = \text{EMA}(\text{MACD Line}, 9)_t$$
  $$\text{Histogram}_t = \text{MACD Line}_t - \text{Signal Line}_t$$
- **Average True Range (ATR)**:
  $$\text{TR}_t = \max(\text{High}_t - \text{Low}_t, |\text{High}_t - \text{Close}_{t-1}|, |\text{Low}_t - \text{Close}_{t-1}|)$$
  $$\text{ATR}_t = \frac{\text{ATR}_{t-1} \times (N-1) + \text{TR}_t}{N}$$
  *(measures market volatility over a 14-day window)*
- **Stochastic Oscillator**:
  $$\%K_t = \frac{\text{Close}_t - \text{Lowest Low}_N}{\text{Highest High}_N - \text{Lowest Low}_N} \times 100$$
  $$\%D_t = \text{SMA}(\%K, 3)_t$$
  *(tracks price relative to High-Low ranges over $N=14$ periods)*
- **On-Balance Volume (OBV)**:
  $$\text{OBV}_t = \text{OBV}_{t-1} + \begin{cases} 
      \text{Volume}_t & \text{if } \text{Close}_t > \text{Close}_{t-1} \\ 
      -\text{Volume}_t & \text{if } \text{Close}_t < \text{Close}_{t-1} \\ 
      0 & \text{otherwise} 
   \end{cases}$$
- **Average Directional Index (ADX)**:
  Measures trend strength using smoothed directional movement indexes ($+DI$, $-DI$, and $DX$) over 14 periods.

---

## 3. Machine Learning Preprocessing (`--prep-ml`)

For machine learning integration (e.g. training models in PyTorch or TensorFlow), enabling `--prep-ml` scales every numerical feature in the aligned DataFrame (except the `date` column):

1. **Standard Z-Score Scaling** (`{col}_standard`):
   $$x_{\text{standard}} = \frac{x - \mu}{\sigma}$$
   *(where $\mu$ is the mean and $\sigma$ is the standard deviation)*
2. **MinMax Scaling** (`{col}_minmax`):
   $$x_{\text{minmax}} = \frac{x - x_{\text{min}}}{x_{\text{max}} - x_{\text{min}}}$$
   *(scales values to fall strictly between `0.0` and `1.0`)*

---

## 4. Markowitz Mean-Variance Portfolio Optimization

When running a multi-asset pipeline, CDG runs a Monte Carlo simulation (10,000 iterations by default) to find optimal allocations:

### Mathematical Engine
1. **Daily Returns**: Computes daily percentage returns for each asset.
2. **Mean & Covariance Matrix**: Calculates average daily returns $\mu_i$ and the asset covariance matrix $\Sigma$ across all non-null values.
3. **Random Portfolios**: Uses a custom parallelized Xorshift random number generator with Rayon to draw asset weights $w$ satisfying $\sum w_i = 1$ and $w_i \ge 0$.
4. **Annualization**: Annualized returns and volatility are scaled using a standard factor:
   $$\text{Trading Days per Year} = 365.0$$
5. **Portfolio Performance**:
   - **Annualized Return**:
     $$R_p = \sum_{i=1}^m w_i \mu_i \times 365$$
   - **Annualized Volatility**:
     $$\sigma_p = \sqrt{w^T \Sigma w \times 365}$$
   - **Sharpe Ratio**:
     $$\text{Sharpe} = \frac{R_p - R_f}{\sigma_p}$$
     *(assuming Risk-Free Rate $R_f = 0.0$)*

### Output Portfolios
- **Max Sharpe Ratio**: The portfolio maximizing expected return per unit of volatility.
- **Minimum Volatility**: The portfolio minimizing overall risk.
