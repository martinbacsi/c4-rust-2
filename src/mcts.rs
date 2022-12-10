use crate::config::{ActionSelection, Exploration, Fpu, MCTSConfig, PolicyNoise};
use crate::connect4::{Game, Outcome};
use crate::policies::Policy;
use std::{time::Instant};

type NodeId = u32;
type ActionId = u8;

impl Into<usize> for Outcome {
    fn into(self) -> usize {
        match self {
            Outcome::Lose(_) => 0,
            Outcome::Draw(_) => 1,
            Outcome::Win(_) => 2,
        }
    }
}

impl Into<[f32; 3]> for Outcome {
    fn into(self) -> [f32; 3] {
        let mut dist = [0.0; 3];
        dist[Into::<usize>::into(self)] = 1.0;
        dist
    }
}

#[derive(Debug)]
struct Node<G: Game<N>, const N: usize> {
    parent: NodeId,            // 4 bytes
    first_child: NodeId,       // 4 bytes
    num_children: u8,          // 1 byte
    game: G,                   // ? bytes
    solution: Option<Outcome>, // 1 byte
    action: ActionId,          // 1 byte
    action_prob: f32,          // 4 bytes
    outcome_probs: [f32; 3],
    num_visits: f32, // 4 bytes
}

impl<G: Game<N>, const N: usize> Node<G, N> {
    fn q(&self) -> f32 {
        (self.outcome_probs[2] - self.outcome_probs[0]) / self.num_visits
    }

    fn unvisited(
        parent: NodeId,
        game: G,
        solution: Option<Outcome>,
        action: u8,
        action_prob: f32,
    ) -> Self {
        Self {
            parent,
            first_child: 0,
            num_children: 0,
            game,
            action,
            solution,
            action_prob,
            outcome_probs: [0.0; 3],
            num_visits: 0.0,
        }
    }

    fn action(&self) -> G::Action {
        (self.action as usize).into()
    }

    #[inline]
    fn is_unvisited(&self) -> bool {
        self.num_children == 0 && self.solution.is_none()
    }

    #[inline]
    fn is_visited(&self) -> bool {
        self.num_children != 0
    }

    #[inline]
    fn is_unsolved(&self) -> bool {
        self.solution.is_none()
    }

    #[inline]
    fn last_child(&self) -> NodeId {
        self.first_child + self.num_children as u32
    }

    #[inline]
    fn mark_visited(&mut self, first_child: NodeId, num_children: u8) {
        self.first_child = first_child;
        self.num_children = num_children;
    }

    #[inline]
    fn mark_solved(&mut self, outcome: Outcome) {
        self.solution = Some(outcome);
    }
}

pub struct MCTS<'a, G: Game<N>, P: Policy<G, N>, const N: usize> {
    root: NodeId,
    offset: NodeId,
    nodes: Vec<Node<G, N>>,
    policy: &'a mut P,
    cfg: MCTSConfig,
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    pub fn exploit(
        explores: usize,
        cfg: MCTSConfig,
        policy: &'a mut P,
        game: G,
        action_selection: ActionSelection,
    ) -> G::Action {
        let mut mcts = Self::with_capacity(explores + 1, cfg, policy, game);
        mcts.explore_n(explores);
        mcts.best_action(action_selection)
    }

    pub fn with_capacity(capacity: usize, cfg: MCTSConfig, policy: &'a mut P, game: G) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(Node::unvisited(0, game, None, 0, 0.0));
        let mut mcts = Self {
            root: 0,
            offset: 0,
            nodes,
            policy,
            cfg,
        };
        let (node_id, outcome_probs, any_solved) = mcts.visit(mcts.root);
        mcts.backprop(node_id, outcome_probs, any_solved);
        //mcts.add_root_noise();
        mcts
    }

    pub fn explore_n(&mut self, n: usize) {

       
        for _ in 0..n {
            // NOTE this is important for value extraction because if root is solved then children might not have any visits
            if self.node(self.root).solution.is_some() {
                break;
            }
            self.explore();
        }
    }

    pub fn explore_until(&mut self, endt: Instant) {
        while Instant::now() < endt {
            // NOTE this is important for value extraction because if root is solved then children might not have any visits
            if self.node(self.root).solution.is_some() {
                break;
            }
            self.explore();
        }
    }


}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    fn next_node_id(&self) -> NodeId {
        self.nodes.len() as NodeId + self.offset
    }

    fn node(&self, node_id: NodeId) -> &Node<G, N> {
        &self.nodes[(node_id - self.offset) as usize]
    }

    fn mut_node(&mut self, node_id: NodeId) -> &mut Node<G, N> {
        &mut self.nodes[(node_id - self.offset) as usize]
    }

    fn children_of(&self, node: &Node<G, N>) -> &[Node<G, N>] {
        &self.nodes
            [(node.first_child - self.offset) as usize..(node.last_child() - self.offset) as usize]
    }

    fn mut_nodes(&mut self, first_child: NodeId, last_child: NodeId) -> &mut [Node<G, N>] {
        &mut self.nodes[(first_child - self.offset) as usize..(last_child - self.offset) as usize]
    }
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    pub fn target_policy(&self, search_policy: &mut [f32; N]) {
        search_policy.fill(0.0);
        let mut total = 0.0;
        let root = self.node(self.root);
        if root.num_visits == 1.0 {
            // assert!(root.solution.is_some());
            match root.solution {
                Some(Outcome::Win(_)) => {
                    for child in self.children_of(root) {
                        let v = if let Some(Outcome::Lose(_)) = child.solution {
                            1.0
                        } else {
                            0.0
                        };
                        search_policy[child.action as usize] = v;
                        total += v;
                    }
                }
                _ => {
                    for child in self.children_of(root) {
                        search_policy[child.action as usize] = 1.0;
                        total += 1.0;
                    }
                }
            }
        } else {
            // assert!(root.num_visits > 1.0);
            for child in self.children_of(root) {
                let v = child.num_visits;
                search_policy[child.action as usize] = v;
                total += v;
            }
        }
        // assert!(total > 0.0, "{:?} {:?}", root.solution, root.num_visits);
        for i in 0..N {
            search_policy[i] /= total;
        }
    }

    pub fn target_q(&self) -> [f32; 3] {
        let root = self.node(self.root);
        match root.solution {
            Some(outcome) => outcome.into(),
            None => {
                let mut outcome_probs = [0.0; 3];
                for i in 0..3 {
                    outcome_probs[i] = root.outcome_probs[i] / root.num_visits;
                }
                outcome_probs
            }
        }
    }
}



impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    pub fn best_action(&self, action_selection: ActionSelection) -> G::Action {
        let root = self.node(self.root);

        let mut best_action = None;
        let mut best_value = None;
        for child in self.children_of(root) {
            let value = match child.solution {
                Some(Outcome::Win(turns)) => Some((0.0, turns as f32)),
                None => match action_selection {
                    ActionSelection::Q => Some((1.0, -child.q())),
                    ActionSelection::NumVisits => Some((1.0, child.num_visits)),
                },
                Some(Outcome::Draw(turns)) => Some((2.0, -(turns as f32))),
                Some(Outcome::Lose(turns)) => Some((3.0, -(turns as f32))),
            };
            if value > best_value {
                best_value = value;
                best_action = Some(child.action());
            }
        }
        best_action.unwrap()
    }

    pub fn solution(&self, action: &G::Action) -> Option<Outcome> {
        let action: usize = (*action).into();
        let action = action as u8;
        let root = self.node(self.root);
        for child in self.children_of(root) {
            if child.action == action {
                return child.solution;
            }
        }
        None
    }
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    fn explore(&mut self) {
        let mut node_id = self.root;
        loop {
            let node = self.node(node_id);
            if let Some(outcome) = node.solution {
                self.backprop(node_id, outcome.into(), true);
                return;
            } else if node.is_unvisited() {
                let (node_id, outcome_probs, any_solved) = self.visit(node_id);
                self.backprop(node_id, outcome_probs, any_solved);
                return;
            } else {
                node_id = self.select_best_child(node);
            }
        }
    }

    fn select_best_child(&self, parent: &Node<G, N>) -> NodeId {
        let mut best_child_id = None;
        let mut best_value = None;
        for child_id in parent.first_child..parent.last_child() {
            let child = self.node(child_id);
            let q = self.exploit_value(parent, child);
            let u = self.explore_value(parent, child);
            let value = Some(q + u);
            if value > best_value {
                best_child_id = Some(child_id);
                best_value = value;
            }
        }
        best_child_id.unwrap()
    }

    fn exploit_value(&self, parent: &Node<G, N>, child: &Node<G, N>) -> f32 {
        if let Some(outcome) = child.solution {
            if self.cfg.select_solved_nodes {
                outcome.reversed().value()
            } else {
                f32::NEG_INFINITY
            }
        } else if child.num_children == 0 {
            match self.cfg.fpu {
                Fpu::Const(value) => value,
                Fpu::ParentQ => parent.q(),
                Fpu::Func(fpu_fn) => (fpu_fn)(),
            }
        } else {
            -child.q()
        }
    }

    fn explore_value(&self, parent: &Node<G, N>, child: &Node<G, N>) -> f32 {
        match self.cfg.exploration {
            Exploration::Uct { c } => {
                let visits = (c * parent.num_visits.ln()).sqrt();
                visits / child.num_visits.sqrt()
            }
            Exploration::PolynomialUct { c } => {
                let visits = parent.num_visits.sqrt();
                c * child.action_prob * visits / (1.0 + child.num_visits)
            }
        }
    }

    fn visit(&mut self, node_id: NodeId) -> (NodeId, [f32; 3], bool) {
        let first_child = self.next_node_id();
        let node = self.node(node_id);
        if let Some(outcome) = node.solution {
            return (node_id, outcome.into(), true);
        }

        let game = node.game.clone();
        let mut num_children = 0;
        let mut any_solved = false;
        for action in game.iter_actions() {
            let mut child_game = game.clone();
            let is_over = child_game.step(&action);
            let solution = if is_over {
                any_solved = true;
                Some(child_game.reward(child_game.player()).into())
            } else {
                None
            };
            let action: usize = action.into();
            let child = Node::unvisited(node_id, child_game, solution, action as u8, 1.0);
            self.nodes.push(child);
            num_children += 1;
        }

        let node = self.mut_node(node_id);
        node.mark_visited(first_child, num_children);
        let first_child = node.first_child;
        let last_child = node.last_child();

        if self.cfg.auto_extend && num_children == 1 {
            return self.visit(first_child);
        } else {
            let (logits, outcome_probs) = self.policy.eval(&game);

            // stable softmax
            let mut max_logit = f32::NEG_INFINITY;
            for child in self.mut_nodes(first_child, last_child) {
                let logit = logits[child.action as usize];
                max_logit = max_logit.max(logit);
                child.action_prob = logit;
            }
            let mut total = 0.0;
            for child in self.mut_nodes(first_child, last_child) {
                child.action_prob = (child.action_prob - max_logit).exp();
                total += child.action_prob;
            }
            for child in self.mut_nodes(first_child, last_child) {
                child.action_prob /= total;
            }

            (node_id, outcome_probs, any_solved)
        }
    }

    fn backprop(&mut self, leaf_node_id: NodeId, mut outcome_probs: [f32; 3], mut solved: bool) {
        let mut node_id = leaf_node_id;
        loop {
            let node = self.node(node_id);
            let parent = node.parent;

            if self.cfg.solve && solved {
                // compute whether all children are solved & best solution so far
                let mut all_solved = true;
                let mut best_solution = node.solution;
                for child in self.children_of(node) {
                    let soln = child.solution.map(|o| o.reversed());
                    all_solved &= soln.is_some();
                    best_solution = best_solution.max(soln);
                }

                let correct_values = self.cfg.correct_values_on_solve;
                let node = self.mut_node(node_id);
                if let Some(Outcome::Win(in_turns)) = best_solution {
                    // at least 1 is a win, so mark this node as a win
                    node.mark_solved(Outcome::Win(in_turns));
                    if correct_values {
                        for i in 0..3 {
                            outcome_probs[i] = -node.outcome_probs[i];
                        }
                        outcome_probs[2] += node.num_visits + 1.0;
                    }
                } else if best_solution.is_some() && all_solved {
                    // all children node's are proven losses or draws
                    let best_outcome = best_solution.unwrap();
                    node.mark_solved(best_outcome);
                    if correct_values {
                        for i in 0..3 {
                            outcome_probs[i] = -node.outcome_probs[i];
                        }
                        if let Outcome::Draw(_) = best_outcome {
                            outcome_probs[1] += node.num_visits + 1.0;
                        } else {
                            outcome_probs[0] += node.num_visits + 1.0;
                        }
                    }
                } else {
                    solved = false;
                }
            }

            let node = self.mut_node(node_id);
            for i in 0..3 {
                node.outcome_probs[i] += outcome_probs[i];
            }
            node.num_visits += 1.0;
            if node_id == self.root {
                break;
            }
            let t = outcome_probs[0];
            outcome_probs[0] = outcome_probs[2];
            outcome_probs[2] = t;
            node_id = parent;
        }
    }
}

