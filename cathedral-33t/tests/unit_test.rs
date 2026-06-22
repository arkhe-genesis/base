use cathedral_arkhe_33t::moe::{MoELayer, HierarchicalRouter, RoutingIndex};
use cathedral_arkhe_33t::config::MoEConfig;
use cathedral_arkhe_33t::tensor::Tensor;

#[test]
fn test_hierarchical_router_route() {
    let router = HierarchicalRouter::new(4096, 8, 128);
    let token = Tensor::randn((2, 128));

    let routing = router.route(&token);

    assert_eq!(routing.len(), 2);
    for indices in routing {
        assert_eq!(indices.len(), 8);
        for index in indices {
            assert!(index.expert_id < 4096);
        }
    }
}

#[test]
fn test_moe_layer_forward() {
    let config = MoEConfig {
        num_experts: 4,
        top_k: 2,
        hidden_size: 16,
        intermediate_size: 64,
        capacity_factor: 1.25,
        load_balancing_loss_coef: 0.01,
    };

    let mut moe = MoELayer::new(&config);
    let x = Tensor::randn((3, 16));

    let (output, aux_loss) = moe.forward(&x);

    assert_eq!(output.shape(), (3, 16));
    assert!(aux_loss >= 0.0);
}
