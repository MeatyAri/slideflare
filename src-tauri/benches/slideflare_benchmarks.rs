use criterion::{criterion_group, criterion_main, Criterion};
use slideflare_lib::incremental::{
    compute_slide_hash, compute_slide_hashes, detect_slide_changes, VecSlideHashes,
};
use slideflare_lib::parser::{
    parse_individual_slide, parse_markdown_with_frontmatter, post_process_asset_paths,
    read_file_as_base64, split_into_sections,
};

const SLIDES_10: &str = r#"---
title: Welcome
---
# Advanced Machine Learning

- Comprehensive overview
- Theory and practice

<div class="info-box">
  <strong>Note:</strong> This course covers modern ML techniques.
</div>

## Course Structure

| Week | Topic | Assignments |
|------|-------|-------------|
| 1 | Introduction | Quiz 1 |
| 2 | Linear Models | HW 1 |
| 3 | Neural Networks | Project |

### Learning Objectives

1. Understand core concepts
2. Implement algorithms
3. Apply to real problems

---

title: Mathematical Foundations
---
## Linear Algebra Review

### Matrix Operations

- Matrix multiplication: $C = AB$
- Transpose: $(A^T)^T = A$
- Inverse: $A A^{-1} = I$

$$\begin{bmatrix} a & b \\ c & d \end{bmatrix}^{-1} = \frac{1}{ad-bc} \begin{bmatrix} d & -b \\ -c & a \end{bmatrix}$$

### Eigenvalues

$$Av = \lambda v$$

Used in PCA and spectral methods.

<span class="highlight">Important:</span> Eigen decompositions are foundational.

---

title: Probability Theory
---
## Bayes Theorem

$$P(A|B) = \frac{P(B|A) P(A)}{P(B)}$$

### Common Distributions

| Distribution | PMF/PDF | Mean | Variance |
|--------------|---------|------|----------|
| Bernoulli | $p^x (1-p)^{1-x}$ | $p$ | $p(1-p)$ |
| Gaussian | $\frac{1}{\sigma\sqrt{2\pi}} e^{-\frac{(x-\mu)^2}{2\sigma^2}}$ | $\mu$ | $\sigma^2$ |
| Poisson | $\frac{\lambda^k e^{-\lambda}}{k!}$ | $\lambda$ | $\lambda$ |

<div class="formula">
$$P(X=k) = \frac{\lambda^k e^{-\lambda}}{k!}$$
</div>

---

title: Linear Regression
---
## Objective Function

$$\min_{w,b} J(w,b) = \min_{w,b} \sum_{i=1}^{n} (y_i - (w^T x_i + b))^2$$

### Gradient Descent Update

$$w := w - \alpha \frac{1}{n} \sum_{i=1}^{n} (h_w(x_i) - y_i) x_i$$

Where $\alpha$ is the learning rate.

$$RSS = \sum_{i=1}^{n}(y_i - \hat{y}_i)^2$$

<span>R-squared measures explained variance.</span>

---

title: Regularization
---
## Ridge Regression (L2)

$$\min_w \sum_{i=1}^{n} (y_i - w^T x_i)^2 + \lambda \|w\|_2^2$$

### Lasso Regression (L1)

$$\min_w \sum_{i=1}^{n} (y_i - w^T x_i)^2 + \lambda \|w\|_1$$

<div class="comparison">
  <ul>
    <li>L2: shrinks weights but doesn't zero them</li>
    <li>L1: sparse solutions, feature selection</li>
  </ul>
</div>

### Elastic Net

$$\min_w \sum_{i=1}^{n} (y_i - w^T x_i)^2 + \lambda_1 \|w\|_1 + \lambda_2 \|w\|_2^2$$

---

title: Classification
---
## Logistic Regression

$$h_w(x) = \sigma(w^T x) = \frac{1}{1 + e^{-w^T x}}$$

### Cross-Entropy Loss

$$J(w) = -\frac{1}{n} \sum_{i=1}^{n} [y_i \log(h_w(x_i)) + (1-y_i) \log(1-h_w(x_i))]$$

<div class="note">
  <strong>Tip:</strong> Use softmax for multi-class problems.
</div>

### Softmax Function

$$P(y=k|x) = \frac{e^{w_k^T x}}{\sum_{j=1}^{K} e^{w_j^T x}}$$

---

title: Neural Networks
---
## Forward Propagation

$$z^{(l)} = W^{(l)} a^{(l-1)} + b^{(l)}$$
$$a^{(l)} = \sigma(z^{(l)})$$

### Backpropagation

$$\frac{\partial J}{\partial W^{(l)}} = \delta^{(l)} (a^{(l-1)})^T$$

$$\delta^{(l)} = (W^{(l)})^T \delta^{(l+1)} \odot \sigma'(z^{(l)})$$

<div class="layers">
  <ul>
    <li>Input layer: $a^{(0)} = x$</li>
    <li>Hidden layers: $a^{(l)}$</li>
    <li>Output layer: $a^{(L)}$</li>
  </ul>
</div>

---

title: Optimization
---
## Advanced Optimizers

### Momentum

$$v := \beta v + \alpha \nabla J(w)$$
$$w := w - v$$

### Adam

$$m := \beta_1 m + (1-\beta_1) \nabla J(w)$$
$$v := \beta_2 v + (1-\beta_2) (\nabla J(w))^2$$
$$\hat{m} = \frac{m}{1-\beta_1^t}, \quad \hat{v} = \frac{v}{1-\beta_2^t}$$
$$w := w - \alpha \frac{\hat{m}}{\sqrt{\hat{v}} + \epsilon}$$

<div class="convergence">
  <strong>Convergence rates:</strong>
  <ul>
    <li>SGD: $O(1/\sqrt{T})$</li>
    <li>Adam: $O(1/T)$</li>
  </ul>
</div>

---

title: Evaluation
---
## Metrics

| Task | Metric | Formula |
|------|--------|---------|
| Regression | MSE | $\frac{1}{n}\sum(y_i - \hat{y}_i)^2$ |
| Classification | Accuracy | $\frac{TP+TN}{TP+TN+FP+FN}$ |
| Ranking | MAP | $\frac{1}{m}\sum \frac{1}{k_i} \sum_{j=1}^{k_i} rel_j$ |

### Cross-Validation

$$CV = \frac{1}{k} \sum_{i=1}^{k} Error(Test_i)$$

<div class="metrics">
  <span>F1 = $\frac{2 \cdot Precision \cdot Recall}{Precision + Recall}$</span>
</div>

---

title: Conclusion
---
## Summary

- Linear models: foundation of ML
- Regularization: prevent overfitting
- Neural networks: handle complex patterns
- Optimization: gradient-based methods

<div class="next-steps">
  <strong>Next Steps</strong>
  <ol>
    <li>Implement algorithms from scratch</li>
    <li>Work on real-world datasets</li>
    <li>Explore deep learning frameworks</li>
  </ol>
</div>

### Questions?

$$f(x) = \int_{-\infty}^{x} p(t) dt$$
"#;

const SLIDES_50: &str = r#"---
title: Welcome
---
# Advanced Machine Learning

<div class="overview">Comprehensive overview of theory and practice.</div>

## Course Structure

| Week | Topic | Assignments |
|------|-------|-------------|
| 1 | Introduction | Quiz 1 |
| 2 | Linear Models | HW 1 |
| 3 | Neural Networks | Project |

<span class="highlight">Duration: 12 weeks</span>

---

title: Mathematical Foundations
---
## Linear Algebra Review

$$A = \begin{bmatrix} a_{11} & a_{12} \\ a_{21} & a_{22} \end{bmatrix}$$

### Matrix Operations

- Multiplication: $C = AB$
- Transpose: $(A^T)^T = A$
- Inverse: $A A^{-1} = I$

<span>Eigen decompositions are foundational.</span>

---

title: Probability Theory
---
## Bayes Theorem

$$P(A|B) = \frac{P(B|A) P(A)}{P(B)}$$

### Distributions

| Type | Distribution | Mean | Variance |
|------|--------------|------|-----------|
| Discrete | Bernoulli | $p$ | $p(1-p)$ |
| Continuous | Gaussian | $\mu$ | $\sigma^2$ |
| Count | Poisson | $\lambda$ | $\lambda$ |

<div class="formula">$$N(\mu, \sigma^2)$$</div>

---

title: Linear Regression
---
## Objective Function

$$\min_{w,b} J(w,b) = \sum_{i=1}^{n} (y_i - w^T x_i - b)^2$$

### Gradient Descent

$$w := w - \alpha \nabla J(w)$$

<span>Learning rate $\alpha$ controls step size.</span>

---

title: Regularization
---
## Ridge (L2)

$$\min_w \|y - Xw\|^2 + \lambda \|w\|_2^2$$

## Lasso (L1)

$$\min_w \|y - Xw\|^2 + \lambda \|w\|_1$$

<div class="comparison">
  <ul>
    <li>Ridge: shrinks coefficients</li>
    <li>Lasso: sparsity</li>
  </ul>
</div>

---

title: Classification
---
## Logistic Regression

$$h_w(x) = \frac{1}{1 + e^{-w^T x}}$$

### Loss Function

$$J(w) = -\frac{1}{n} \sum [y \log h + (1-y) \log(1-h)]$$

<div class="note">Binary and multi-class variants.</div>

---

title: Neural Networks
---
## Forward Pass

$$z^{(l)} = W^{(l)} a^{(l-1)} + b^{(l)}$$
$$a^{(l)} = \sigma(z^{(l)})$$

### Backward Pass

$$\frac{\partial J}{\partial W^{(l)}} = \delta^{(l)} (a^{(l-1)})^T$$

<span>Chain rule enables learning.</span>

---

title: Optimization
---
## Gradient Descent

$$w_{t+1} = w_t - \alpha \nabla J(w_t)$$

### Momentum

$$v_{t+1} = \beta v_t + \alpha \nabla J(w_t)$$
$$w_{t+1} = w_t - v_{t+1}$$

<div class="optimizer">Adam combines momentum and scaling.</div>

---

title: Evaluation Metrics
---
## Regression Metrics

| Metric | Formula |
|--------|---------|
| MSE | $\frac{1}{n}\sum(y_i - \hat{y}_i)^2$ |
| MAE | $\frac{1}{n}\sum|y_i - \hat{y}_i|$ |
| $R^2$ | $1 - \frac{SS_{res}}{SS_{tot}}$ |

<span>Lower is better for MSE/MAE.</span>

---

title: Evaluation Metrics 2
---
## Classification Metrics

| Metric | Formula |
|--------|---------|
| Accuracy | $\frac{TP+TN}{TP+TN+FP+FN}$ |
| Precision | $\frac{TP}{TP+FP}$ |
| Recall | $\frac{TP}{TP+FN}$ |

<div class="f1score">$$F_1 = \frac{2 Precision \cdot Recall}{Precision + Recall}$$</div>

---

title: Support Vector Machines
---
## Maximum Margin

$$\min_{w,b} \frac{1}{2} \|w\|^2$$

Subject to: $y_i(w^T x_i + b) \geq 1$

### Kernel Trick

$$K(x, x') = \phi(x)^T \phi(x')$$

<span>Gaussian/RBF kernel is common.</span>

---

title: Decision Trees
---
## Information Gain

$$IG = H(parent) - \sum_{i} \frac{N_i}{N} H(child_i)$$

### Entropy

$$H = -\sum p_i \log_2 p_i$$

<div class="gini">$$Gini = 1 - \sum p_i^2$$</div>

---

title: Ensemble Methods
---
## Bagging

$$\hat{f}(x) = \frac{1}{B} \sum_{b=1}^{B} \hat{f}_b(x)$$

### Random Forest

- Bagging + feature subsampling
- Reduces variance
- Handles high-dimensional data

<span>Averages multiple weak learners.</span>

---

title: Boosting
---
## AdaBoost

$$w_{i}^{(t+1)} = w_i^{(t)} \exp(-\alpha_t y_i h_t(x_i))$$

### Gradient Boosting

$$\min_{\gamma} \sum_{i=1}^{n} L(y_i, F_{t-1}(x_i) + \gamma h_t(x_i))$$

<div class="sequential">Sequential error correction.</div>

---

title: Dimensionality Reduction
---
## PCA

$$z = U^T x$$

### PCA Objective

$$\max_{U} \text{tr}(U^T \Sigma U)$$

Subject to: $U^T U = I$

<span>Principal components maximize variance.</span>

---

title: Clustering
---
## K-Means

$$\min_{C} \sum_{k=1}^{K} \sum_{x \in C_k} \|x - \mu_k\|^2$$

### Algorithm

1. Initialize centroids
2. Assign points to nearest
3. Update centroids
4. Repeat until convergence

<div class="iterative">Iterative optimization.</div>

---

title: Clustering 2
---
## Hierarchical Clustering

- Agglomerative (bottom-up)
- Divisive (top-down)

### Linkage Methods

| Method | Formula |
|--------|---------|
| Single | $\min d(x,y)$ |
| Complete | $\max d(x,y)$ |
| Average | $\frac{1}{|A||B|} \sum d(x,y)$ |

<span>Dendrograms visualize hierarchy.</span>

---

title: Feature Engineering
---
## Numeric Features

- Scaling: $x' = \frac{x - \mu}{\sigma}$
- Normalization: $x' = \frac{x - x_{min}}{x_{max} - x_{min}}$

### Categorical Encoding

- One-hot: $\text{OneHot}(x)$
- Label: $\text{Label}(x)$

<div class="transformation">$$x_{new} = T(x)$$</div>

---

title: Feature Engineering 2
---
## Text Features

### Bag of Words

$$\phi(x)_i = \text{count}(word_i \in x)$$

### TF-IDF

$$tfidf(t,d) = tf(t,d) \cdot idf(t)$$

<span>Word embeddings capture semantics.</span>

---

title: Model Selection
---
## Cross-Validation

$$CV = \frac{1}{k} \sum_{i=1}^{k} Error(Test_i)$$

### K-Fold

- k=5 or k=10 common
- Stratified for classification

<div class="validation">$$Error = \frac{1}{n} \sum L(y_i, \hat{y}_i)$$</div>

---

title: Bias-Variance
---
## Decomposition

$$E[(y - \hat{f}(x))^2] = \text{Bias}^2 + \text{Variance} + \sigma^2$$

### Trade-off

- High bias: underfitting
- High variance: overfitting

<span>$$\text{Bias}(\hat{f}) = E[\hat{f}] - f$$</span>

---

title: Learning Curves
---
## Analysis

$$\text{Error}_{train} = f(n_{train})$$
$$\text{Error}_{test} = g(n_{train})$$

### Diagnosing

| Train Error | Test Error | Problem |
|-------------|------------|---------|
| Low | High | Overfitting |
| High | High | Underfitting |

<div class="diagnosis">More data or simpler model needed.</div>

---

title: Hyperparameter Tuning
---
## Grid Search

$$\min_{\lambda \in \Lambda} CV(\lambda)$$

### Random Search

- Sample from distributions
- More efficient than grid

<span>Bayesian optimization is state-of-art.</span>

---

title: Regularization Effects
---
## Path Diagrams

$$\lambda \rightarrow \text{coefficients} \rightarrow \text{model}$$

### L1 vs L2

| Property | L1 (Lasso) | L2 (Ridge) |
|----------|------------|-----------|
| Sparsity | Yes | No |
| Solution | Sparse | Dense |
| Computation | Convex | Convex |

<div class="visualization">Coefficient paths vs regularization.</div>

---

title: Deep Learning Basics
---
## Neural Layers

$$a^{(l)} = \sigma(W^{(l)} a^{(l-1)} + b^{(l)})$$

### Common Activations

- ReLU: $\max(0, z)$
- Sigmoid: $\frac{1}{1+e^{-z}}$
- Tanh: $\tanh(z)$

<span>Depth enables hierarchical representations.</span>

---

title: Convolutional Networks
---
## Convolution

$$(f * g)(t) = \int f(\tau) g(t - \tau) d\tau$$

### 2D Convolution

$$y_{i,j} = \sum_k \sum_l w_{k,l} x_{i+k, j+l}$$

<div class="filter">Filters detect local patterns.</div>

---

title: Convolutional Networks 2
---
## Pooling

$$\text{Pool}(S) = \max_{i \in S} x_i$$

### Types

- Max pooling
- Average pooling
- Global pooling

<span>Reduces spatial dimensions.</span>

---

title: Recurrent Networks
---
## RNN

$$h_t = \sigma(W h_{t-1} + U x_t)$$

### LSTM

$$f_t = \sigma(W_f h_{t-1} + U_f x_t)$$
$$i_t = \sigma(W_i h_{t-1} + U_i x_t)$$

<div class="temporal">Sequential modeling capability.</div>

---

title: Attention Mechanism
---
## Attention

$$\text{Attention}(Q, K, V) = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}}\right) V$$

### Self-Attention

$$A = \text{softmax}\left(\frac{QK^T}{\sqrt{d}}\right) V$$

<span>Transformer architecture base.</span>

---

title: Generative Models
---
## VAE

$$\min_{\phi, \theta} \mathbb{E}_q[\log p_\theta(x|z)] + D_{KL}(q_\phi(z|x) || p(z))$$

### GAN

$$\min_G \max_D \mathbb{E}_x[\log D(x)] + \mathbb{E}_z[\log(1 - D(G(z)))]$$

<div class="generative">Generate new data samples.</div>

---

title: Reinforcement Learning
---
## Bellman Equation

$$V(s) = \max_a \left[ R(s,a) + \gamma \sum_{s'} P(s'|s,a) V(s') \right]$$

### Q-Learning

$$Q(s,a) \leftarrow Q(s,a) + \alpha [r + \gamma \max_{a'} Q(s',a') - Q(s,a)]$$

<div class="rl">Learn optimal policy.</div>

---

title: Multi-Armed Bandits
---
## Explore-Exploit

$$\text{Regret} = \mathbb{E}\left[\sum_{t=1}^{T} r_t\right] - \max_a \mathbb{E}\left[\sum_{t=1}^{T} r_t(a)\right]$$

### UCB

$$a_t = \arg\max_a \left[ \bar{\mu}_a + \sqrt{\frac{2\log t}{N_a}} \right]$$

<span>Balance exploration and exploitation.</span>

---

title: Online Learning
---
## Update Rule

$$\theta_{t+1} = \theta_t - \eta_t \nabla L(\theta_t; x_t, y_t)$$

### Online Gradient Descent

$$w_{t+1} = w_t - \eta_t (\hat{y}_t - y_t) x_t$$

<span>Streaming data scenarios.</span>

---

title: Transfer Learning
---
## Feature Transfer

$$\theta^* = \arg\min_{\theta} \sum_{(x,y) \in D_{target}} L(f_\theta(x), y) + \lambda R(\theta)$$

### Fine-Tuning

- Freeze early layers
- Finetune later layers
- Full finetuning as needed

<span>Knowledge from source to target.</span>

---

title: Semi-Supervised Learning
---
## Label Propagation

$$Y_u = P_{uu}^{-1} P_{ul} Y_l$$

### Consistency Regularization

$$\min_\theta \mathbb{E}_{x \sim D}[ \| f_\theta(x) - f_\theta(x \cdot \mathcal{T}) \|^2 ]$$

<div class="semi">Leverage unlabeled data.</div>

---

title: Meta Learning
---
## MAML

$$\theta^* = \arg\min_{\theta} \sum_{\mathcal{T}_i} L_{\mathcal{T}_i}(f_{\theta - \alpha \nabla_\theta L_{\mathcal{T}_i}(f_\theta)})$$

### Few-Shot Learning

- N-way K-shot classification
- Learn to learn

<span>Rapid adaptation to new tasks.</span>

---

title: Uncertainty Estimation
---
## Bayesian Neural Networks

$$p(w|D) = \frac{p(D|w) p(w)}{p(D)}$$

### Dropout as Bayesian

$$\hat{y} = \frac{1}{T} \sum_{t=1}^{T} f(x, w_t)$$

<span>Quantify prediction uncertainty.</span>

---

title: Interpretability
---
## Feature Importance

$$I_j = \sum_{t=1}^{T} \mathbb{1}\{j \in \text{split at level } t\}$$

### SHAP Values

$$\phi_i = \sum_{S \subseteq F \setminus \{i\}} \frac{|S|!(|F|-|S|-1)!}{|F|!} [f(S \cup \{i\}) - f(S)]$$

<div class="解释">Model explanations for stakeholders.</div>

---

title: Fairness
---
## Demographic Parity

$$P(\hat{Y}=1|A=0) = P(\hat{Y}=1|A=1)$$

### Equalized Odds

$$P(\hat{Y}=1|Y=A=0,Y) = P(\hat{Y}=1|Y=A=1,Y)$$

<span>Machine learning fairness considerations.</span>

---

title: Adversarial Examples
---
## Attack

$$x' = x + \epsilon \cdot \text{sign}(\nabla_x J(\theta, x, y))$$

### Defense

- Adversarial training
- Input preprocessing
- Certified robustness

<div class="security">Robustness against perturbations.</div>

---

title: Efficiency
---
## Model Compression

- Pruning: Remove low-magnitude weights
- Quantization: Reduce bit precision
- Knowledge distillation

###.distillation

$$\min_\theta L_{student}(\theta) + \lambda L_{distill}}(T, \theta)$$

<span>Small models from large ones.</span>

---

title: Distributed Training
---
## Data Parallelism

$$\nabla_\theta L = \frac{1}{n} \sum_{i=1}^{n} \nabla_\theta \ell(x_i, y_i; \theta)$$

### Model Parallelism

$$f(x) = f_k(\cdots f_2(f_1(x)))$$

<div class="parallel">Multi-GPU training.</div>

---

title: AutoML
---
## Neural Architecture Search

$$\max_{a \in \mathcal{A}} \text{ACC}(a)$$

### Search Space

- Number of layers
- Connection types
- Operations

<span>Automate model selection.</span>

---

title: Conclusion
---
## Summary

<div class="key-points">
  <ul>
    <li>Foundation: Linear models, regularization</li>
    <li>Deep learning: Neural networks, attention</li>
    <li>Practice: Engineering, deployment</li>
    <li>Future: Research directions</li>
  </ul>
</div>

### What Next?

1. Implement projects
2. Read recent papers
3. Contribute to open source

<span>Questions welcome!</span>

---

title: Practical Applications
---
## Real World Examples

### Computer Vision

| Task | Model | Accuracy |
|------|-------|---------|
| Classification | ResNet | 76.3% |
| Detection | YOLO | 57.9% |
| Segmentation | U-Net | 89.2% |

<span>Transfer learning accelerates development.</span>

---

title: Natural Language Processing
---
## Transformers

$$Attention(Q, K, V) = softmax\left(\frac{QK^T}{\sqrt{d_k}}\right) V$$

### BERT

- Bidirectional encoding
- Masked language modeling
- Next sentence prediction

$$L_{MLM} = -\sum_{i=1}^{N} \log P(w_i | context)$$

<span>Pre-training + fine-tuning paradigm.</span>

---

title: Time Series
---
## Forecasting

$$\hat{y}_{t+1} = f(y_t, y_{t-1}, ..., y_{t-k})$$

### ARIMA

$$y_t = \phi_1 y_{t-1} + ... + \phi_p y_{t-p} + \epsilon_t$$

<div class="stationarity">
  <strong>Stationarity check:</strong> ADF test for unit roots.
</div>

---

title: Graph Neural Networks
---
## Message Passing

$$h_v^{(k)} = \text{Update}\left(h_v^{(k-1)}, \sum_{u \in N(v)} Message(h_u^{(k-1)}, h_v^{(k-1)})\right)$$

### Applications

- Social networks
- Molecule analysis
- Knowledge graphs

<span>Geometric deep learning.</span>

---

title: Recommender Systems
---
## Matrix Factorization

$$\min_{U, V} \sum_{(u,i) \in R} (r_{ui} - u_i^T v_j)^2 + \lambda(\|U\|^2 + \|V\|^2)$$

### Neural Recommenders

$$y_{ui} = f(user_i, item_j, context)$$

<div class="cold-start">Solutions for cold-start problems.</div>

---

title: Final Remarks
---
## Takeaways

<div class="summary">
  <ul>
    <li>Foundations matter: Linear algebra, probability, optimization</li>
    <li>Modern methods: Deep learning, attention, transformers</li>
    <li>Engineering: Feature engineering, model selection, deployment</li>
    <li>Ethics: Fairness, interpretability, robustness</li>
  </ul>
</div>

### Keep Learning

$$\text{Learning}_{continuous} = \text{Reading} + \text{Practice} + \text{Collaboration}$$

<span>Thank you for your attention!</span>
"#;

const SLIDES_20_SOME_MEDIA: &str = r#"---
title: Slide 1
---
# Presentation

Welcome to this presentation.

![Sample Image](../examples/images/sample-image.png)

Some content here.

---

title: Slide 2
---
## More Content

This slide has **bold** and *italic* text.

$$E = mc^2$$

No media here.

---

title: Slide 3
---
## List of Things

- Item 1
- Item 2
- Item 3

<div>
  <img src="../examples/images/sample-image.png" alt="Image">
</div>

---

title: Slide 4
---
## Code Example

```python
def hello():
    print("Hello World")
```

No media.

---

title: Slide 5
---
## Video Section

<video src="../examples/videos/sample-video.mp4" controls></video>

Great video content!

---

title: Slide 6
---
## Text Only

$$ \int_0^1 x^2 dx = \frac{1}{3} $$

Just math and text.

---

title: Slide 7
---
## Another Image

![Another Image](../examples/images/sample-image.png)

With caption.

---

title: Slide 8
---
## Summary

| Column A | Column B |
|----------|----------|
| Value 1 | Value 2 |

No media.

---

title: Slide 9
---
## More Math

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$$

And text.

---

title: Slide 10
---
## Conclusion

- Point 1
- Point 2

![Image Here](../examples/images/sample-image.png)

---

title: Slide 11
---
## Details

<div>
  <p>Some HTML content</p>
</div>

Just text.

---

title: Slide 12
---
## Additional Info

$$a^2 + b^2 = c^2$$

No media here.

---

title: Slide 13
---
## Media Slide

<video src="../examples/videos/sample-video.mp4" controls muted></video>

Watch this!

---

title: Slide 14
---
## Key Points

1. First
2. Second

Text only.

---

title: Slide 15
---
## Important

$$\lambda = \frac{1}{\sigma} \sqrt{\frac{\sum(x_i - \bar{x})^2}{n}}$$

And content.

---

title: Slide 16
---
## Image Again

![Image Again](../examples/images/sample-image.png)

With more text.

---

title: Slide 17
---
## Final Thoughts

- Final point 1
- Final point 2

No media.

---

title: Slide 18
---
## More Content

$$f(x) = \begin{cases} x & x > 0 \\ 0 & x \leq 0 \end{cases}$$

Just math.

---

title: Slide 19
---
## Before End

![Last Image](../examples/images/sample-image.png)

Almost done.

---

title: Slide 20
---
## End

Thank you!

$$\text{The End}$$
"#;

const SLIDES_20_EVERY_MEDIA: &str = r#"---
title: Slide 1
---
# Welcome

![Image 1](../examples/images/sample-image.png)

$$E = mc^2$$

---

title: Slide 2
---
## Second Slide

<video src="../examples/videos/sample-video.mp4" controls></video>

Some text.

---

title: Slide 3
---
## Third Slide

![Image Here](../examples/images/sample-image.png)

More content.

---

title: Slide 4
---
## Fourth Slide

$$\int_0^1 x^2 dx$$

<div>
  <img src="../examples/images/sample-image.png" alt="Slide 4">
</div>

---

title: Slide 5
---
## Fifth Slide

![Slide 5 Image](../examples/images/sample-image.png)

$$f(x) = x^2$$

---

title: Slide 6
---
## Sixth Slide

<video src="../examples/videos/sample-video.mp4" controls muted></video>

$$\sigma = \sqrt{\frac{1}{n} \sum x_i^2}$$

---

title: Slide 7
---
## Seventh Slide

![Sample](../examples/images/sample-image.png)

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$$

---

title: Slide 8
---
## Eighth Slide

<div>
  <img src="../examples/images/sample-image.png" alt="Eight">
</div>

$$\lambda$$

---

title: Slide 9
---
## Ninth Slide

![Nine](../examples/images/sample-image.png)

$$a^2 + b^2 = c^2$$

---

title: Slide 10
---
## Tenth Slide

<video src="../examples/videos/sample-video.mp4" controls></video>

$$\text{Ten} = 10$$

---

title: Slide 11
---
## Eleventh

![Eleven Image](../examples/images/sample-image.png)

$$\mu = \frac{1}{n} \sum x_i$$

---

title: Slide 12
---
## Twelfth

<div>
  <video src="../examples/videos/sample-video.mp4" controls muted></video>
</div>

$$N(0, 1)$$

---

title: Slide 13
---
## Thirteenth

![13](../examples/images/sample-image.png)

$$\alpha + \beta = \gamma$$

---

title: Slide 14
---
## Fourteenth

$$\hat{y} = w^T x + b$$

<img src="../examples/images/sample-image.png" alt="14">

---

title: Slide 15
---
## Fifteenth

![15](../examples/images/sample-image.png)

$$J(w) = \frac{1}{2} \|y - Xw\|^2$$

---

title: Slide 16
---
## Sixteenth

<video src="../examples/videos/sample-video.mp4" controls></video>

$$\nabla J(w)$$

---

title: Slide 17
---
## Seventeenth

$$\sigma(z) = \frac{1}{1 + e^{-z}}$$

![17](../examples/images/sample-image.png)

---

title: Slide 18
---
## Eighteenth

<div>
  <img src="../examples/images/sample-image.png" alt="18">
</div>

$$\delta^{(l)} = (W^{(l)})^T \delta^{(l+1)} \odot \sigma'(z^{(l)})$$

---

title: Slide 19
---
## Nineteenth

![19](../examples/images/sample-image.png)

$$P(y|x) = \text{softmax}(W x + b)$$

---

title: Slide 20
---
## Twentieth

<video src="../examples/videos/sample-video.mp4" controls></video>

$$\text{Thanks for watching!}$$
"#;

fn bench_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("split");

    group.bench_function("10_slides", |b| {
        b.iter(|| split_into_sections(SLIDES_10));
    });

    group.bench_function("50_slides", |b| {
        b.iter(|| split_into_sections(SLIDES_50));
    });

    group.finish();
}

fn bench_parse_individual(c: &mut Criterion) {
    let single_section = r#"---
title: Single Slide
---
# Hello World

This is a test slide with some **bold** and *italic* text.

- Item 1
- Item 2
"#;

    let latex_section = r#"---
title: Math Slide
---
## Equations

Inline: $x^2 + y^2 = z^2$

Display:
$$\int_0^1 x^2 dx = \frac{1}{3}$$
"#;

    let mut group = c.benchmark_group("parse_individual_slide");

    group.bench_function("simple", |b| {
        b.iter(|| parse_individual_slide(single_section, ""));
    });

    group.bench_function("with_latex", |b| {
        b.iter(|| parse_individual_slide(latex_section, ""));
    });

    group.finish();
}

fn bench_parse_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_full");

    group.bench_function("10_slides", |b| {
        b.iter(|| parse_markdown_with_frontmatter(SLIDES_10, ""));
    });

    group.bench_function("50_slides", |b| {
        b.iter(|| parse_markdown_with_frontmatter(SLIDES_50, ""));
    });

    group.finish();
}

fn bench_parse_media(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_media");

    let examples_dir = env!("CARGO_MANIFEST_DIR");
    let base_path = format!("{}/../examples", examples_dir);

    group.bench_function("20_slides_some_media", |b| {
        b.iter(|| parse_markdown_with_frontmatter(SLIDES_20_SOME_MEDIA, &base_path));
    });

    group.bench_function("20_slides_every_media", |b| {
        b.iter(|| parse_markdown_with_frontmatter(SLIDES_20_EVERY_MEDIA, &base_path));
    });

    group.finish();
}

fn bench_asset_processing(c: &mut Criterion) {
    let html_with_images = r#"<div>
<img src="../examples/images/sample-image.png" alt="test">
<p>Some text</p>
</div>"#;

    let html_with_video = r#"<div>
<video src="../examples/videos/sample-video.mp4" controls></video>
<p>Watch this</p>
</div>"#;

    let examples_dir = env!("CARGO_MANIFEST_DIR");
    let base_path = format!("{}/../examples", examples_dir);

    let mut group = c.benchmark_group("asset_processing");

    group.bench_function("resolve_image_paths", |b| {
        b.iter(|| post_process_asset_paths(html_with_images, &base_path));
    });

    group.bench_function("resolve_video_paths", |b| {
        b.iter(|| post_process_asset_paths(html_with_video, &base_path));
    });

    group.finish();
}

fn bench_asset_real_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_real_files");

    let examples_dir = env!("CARGO_MANIFEST_DIR");

    group.bench_function("read_image_3mb", |b| {
        let img_path = format!("{}/../examples/images/sample-image.png", examples_dir);
        b.iter(|| read_file_as_base64(&img_path));
    });

    group.bench_function("read_video_461kb", |b| {
        let video_path = format!("{}/../examples/videos/sample-video.mp4", examples_dir);
        b.iter(|| read_file_as_base64(&video_path));
    });

    group.finish();
}

fn bench_hash(c: &mut Criterion) {
    let single_slide = r#"---
title: Test Slide
---
# Content here
Some text with **formatting** and $math$.
"#;

    let mut group = c.benchmark_group("hash");

    group.bench_function("single_slide", |b| {
        b.iter(|| compute_slide_hash(single_slide));
    });

    group.bench_function("full_document_10_slides", |b| {
        b.iter(|| compute_slide_hashes(SLIDES_10));
    });

    group.bench_function("full_document_50_slides", |b| {
        b.iter(|| compute_slide_hashes(SLIDES_50));
    });

    group.finish();
}

fn bench_diff(c: &mut Criterion) {
    let hashes_10 = compute_slide_hashes(SLIDES_10).unwrap();
    let hashes_50 = compute_slide_hashes(SLIDES_50).unwrap();

    let mut group = c.benchmark_group("diff");

    group.bench_function("identical_10_slides", |b| {
        let h1 = VecSlideHashes::create_from(hashes_10.data.clone());
        let h2 = VecSlideHashes::create_from(hashes_10.data.clone());
        b.iter(|| detect_slide_changes(&h1, &h2));
    });

    group.bench_function("identical_50_slides", |b| {
        let h1 = VecSlideHashes::create_from(hashes_50.data.clone());
        let h2 = VecSlideHashes::create_from(hashes_50.data.clone());
        b.iter(|| detect_slide_changes(&h1, &h2));
    });

    group.bench_function("few_changes_50_slides", |b| {
        let mut modified = hashes_50.data.clone();
        modified[10] = 99999999;
        modified[20] = 88888888;
        modified[30] = 77777777;
        let h1 = VecSlideHashes::create_from(hashes_50.data.clone());
        let h2 = VecSlideHashes::create_from(modified);
        b.iter(|| detect_slide_changes(&h1, &h2));
    });

    group.bench_function("major_change_50_slides", |b| {
        let modified: Vec<u32> = (0..50)
            .map(|i| {
                if i < 40 {
                    i as u32 + 10000
                } else {
                    hashes_50.data[i]
                }
            })
            .collect();
        let h1 = VecSlideHashes::create_from(hashes_50.data.clone());
        let h2 = VecSlideHashes::create_from(modified);
        b.iter(|| detect_slide_changes(&h1, &h2));
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_split(c);
    bench_parse_individual(c);
    bench_parse_full(c);
    bench_parse_media(c);
    bench_asset_processing(c);
    bench_asset_real_files(c);
    bench_hash(c);
    bench_diff(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
