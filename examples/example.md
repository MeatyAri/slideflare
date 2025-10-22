---
bg_color: bg-indigo-700
text_color: text-white text-center
title: Model Evaluation
---

# Mermaid Graph Example

<div class="mermaid">
graph TD
    A[Data] --> B[Preprocessing]
    B --> C[Model Training]
    C --> D[Validation]
    D --> E{Accuracy > 90%?}
    E -->|Yes| F[Deploy Model]
    E -->|No| G[Adjust Parameters]
    G --> C
</div>

---
bg_color: bg-pink-600
text_color: text-white text-center
title: Decision Tree Example
---

# yo

## hey sup,


<div class="mermaid">
pie showData
    title Distribution of Data Points
    "Category A": 42.86
    "Category B": 28.57
    "Category C": 28.57
</div>

---
bg_color: bg-blue-800
text_color: text-white
title: Introduction to Machine Learning
---

# Welcome to Machine Learning

- [ ] Understand the core concepts
- [x] Explore real-world applications
- [ ] Get hands-on with examples

Machine Learning enables systems to learn patterns from data.

## Let’s get started!

---
bg_color: bg-teal-600
text_color: text-white text-center
title: Types of Learning
---

Supervised, unsupervised, and reinforcement learning are the three main paradigms.

## Categories

- **Supervised**: Learn mapping $(x \mapsto y)$ from labeled data
- **Unsupervised**: Discover hidden structure in unlabeled data
- **Reinforcement**: Agent learns by interacting and receiving rewards

---
bg_color: bg-green-700
text_color: text-white
title: Linear Regression
---

Linear regression fits a line to data by minimizing the sum of squared errors.

$$
\min_{w,b}\; J(w,b)
\;=\;
\min_{w,b}\;\sum_{i=1}^{n}\bigl(y_i - (w x_i + b)\bigr)^{2}
$$

## Look at this subscript: $w_{j}$

---
bg_color: bg-yellow-500
text_color: text-black text-center
title: Regularization
---

Regularization helps prevent overfitting by adding a penalty term.

- **L2 (Ridge)** penalty: $\lambda \sum_{j=1}^{p} w_{j}^{2}$
- **L1 (Lasso)** penalty: $\lambda \sum_{j=1}^{p} |w_{j}|$

## Trade-off: bias vs. variance

---
bg_color: bg-purple-600
text_color: text-white
title: Neural Networks
---

A neural network is composed of layers of interconnected neurons.

$$
a^{(l)} = \sigma\bigl(W^{(l)} a^{(l-1)} + b^{(l)}\bigr)
$$

## Activation functions

- Sigmoid: $\sigma(z) = \frac{1}{1 + e^{-z}}$
- ReLU: $\operatorname{ReLU}(z) = \max(0, z)$
- Tanh: $\tanh(z) = \frac{e^{z} - e^{-z}}{e^{z} + e^{-z}}$

---
bg_color: bg-gray-700
text_color: text-white text-center
title: Sample Image
---

<div class="card">

## Rust Crab Photo
Below is a striking photograph of a rust‑colored crab, highlighting its textured shell and vivid coloration.

![Rust Crab](./images/sample-image.png)

</div>

---
bg_color: bg-red-600
text_color: text-white text-center
title: Sample Video
---

<div class="card">

## Rickroll Surprise

<video src="./videos/sample-video.mp4" autoplay loop></video>

A playful nod to internet culture.

</div>

---
