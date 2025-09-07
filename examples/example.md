---
bgColor: bg-blue-800
textColor: text-white
title: Introduction to Machine Learning
---

# Welcome to Machine Learning

- [ ] Understand the core concepts
- [x] Explore real-world applications
- [ ] Get hands-on with examples

Machine Learning enables systems to learn patterns from data.

## Let’s get started!

---
bgColor: bg-teal-600
textColor: text-white text-center
title: Types of Learning
---

Supervised, unsupervised, and reinforcement learning are the three main paradigms.

## Categories

- **Supervised**: Learn mapping $(x \mapsto y)$ from labeled data
- **Unsupervised**: Discover hidden structure in unlabeled data
- **Reinforcement**: Agent learns by interacting and receiving rewards

---
bgColor: bg-green-700
textColor: text-white
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
bgColor: bg-yellow-500
textColor: text-black text-center
title: Regularization
---

Regularization helps prevent overfitting by adding a penalty term.

- **L2 (Ridge)** penalty: $\lambda \sum_{j=1}^{p} w_{j}^{2}$
- **L1 (Lasso)** penalty: $\lambda \sum_{j=1}^{p} |w_{j}|$

## Trade-off: bias vs. variance

---
bgColor: bg-purple-600
textColor: text-white
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
bgColor: bg-gray-700
textColor: text-white text-center
title: Sample Image
---

![Sample Image](./images/sample-image.png)

---
bgColor: bg-red-600
textColor: text-white text-center
title: Sample Video
---

<video src="./videos/sample-video.mp4" controls></video>

---
